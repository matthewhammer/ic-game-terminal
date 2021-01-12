extern crate delay;
extern crate futures;
extern crate ic_agent;
extern crate ic_types;
extern crate icmt;
extern crate num_traits;
extern crate sdl2;
extern crate serde;
#[macro_use]
extern crate log;
extern crate clap;
extern crate env_logger;

extern crate structopt;
use structopt::StructOpt;

use candid::Decode;
use ic_agent::Agent;
use ic_types::Principal;

use candid::Nat;
use chrono::prelude::*;
use delay::Delay;
use sdl2::event::Event as SysEvent; // not to be confused with our own definition
use sdl2::event::WindowEvent;
use sdl2::keyboard::Keycode;
use sdl2::render::{Canvas, RenderTarget};
use sdl2::surface::Surface;
use std::io;
use std::sync::mpsc;
use std::time::Duration;
use tokio::task;

use icmt::{
    cli::*,
    draw::*,
    error::*,
    keyboard,
    types::{event, graphics, nat_ceil, skip_event, ServiceCall, UserInfoCli},
    write::write_gifs,
};

fn init_log(level_filter: log::LevelFilter) {
    use env_logger::{Builder, WriteStyle};
    let mut builder = Builder::new();
    builder
        .filter(None, level_filter)
        .write_style(WriteStyle::Always)
        .init();
}

const RETRY_PAUSE: Duration = Duration::from_millis(100);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

pub fn create_agent(url: &str) -> IcmtResult<Agent> {
    //use ring::signature::Ed25519KeyPair;
    use ic_agent::agent::AgentConfig;
    use ring::rand::SystemRandom;

    let rng = SystemRandom::new();
    let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)?;
    let key_pair = ring::signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())?;
    let ident = ic_agent::identity::BasicIdentity::from_key_pair(key_pair);
    let agent = Agent::new(AgentConfig {
        identity: Box::new(ident),
        url: format!("http://{}", url),
        ..AgentConfig::default()
    })?;
    Ok(agent)
}

fn translate_system_event(
    video_subsystem: &sdl2::VideoSubsystem,
    event: &SysEvent,
) -> Option<event::Event> {
    match event {
        SysEvent::ClipboardUpdate { .. } => {
            let text = match video_subsystem.clipboard().clipboard_text() {
                Ok(text) => text,
                Err(text) => format!("error: {}", text),
            };
            Some(event::Event::ClipBoard(text))
        }
        SysEvent::Window {
            win_event: WindowEvent::SizeChanged(w, h),
            ..
        } => {
            let dim = graphics::Dim {
                width: Nat::from(*w as u64),
                height: Nat::from(*h as u64),
            };
            Some(event::Event::WindowSize(dim))
        }
        SysEvent::Quit { .. }
        | SysEvent::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => Some(event::Event::Quit),
        SysEvent::KeyDown {
            keycode: Some(ref kc),
            keymod,
            ..
        } => match keyboard::translate_event(kc, keymod) {
            Some(ev) => Some(event::Event::KeyDown(vec![ev])),
            None => None,
        },
        _ => None,
    }
}

async fn do_redraw<'a, T1: RenderTarget>(
    cli: &CliOpt,
    window_dim: &graphics::Dim,
    window_canvas: &mut Canvas<T1>,
    file_canvas: &mut Canvas<Surface<'a>>,
    bmp_paths: &mut Vec<String>,
    data: &graphics::Result,
) -> IcmtResult<()> {
    if !cli.no_window {
        draw(window_canvas, window_dim, data).await?;
    }
    if !cli.no_capture {
        draw(file_canvas, window_dim, data).await?;
        let path = format!(
            "{}/screen-{}x{}-{}.bmp",
            cli.capture_output_path,
            window_dim.width,
            window_dim.height,
            Local::now().to_rfc3339()
        );
        file_canvas.surface().save_bmp(&path)?;
        bmp_paths.push(path);
    }
    Ok(())
}

async fn do_view_task(
    cfg: ConnectCfg,
    remote_in: mpsc::Receiver<Option<(graphics::Dim, Vec<event::EventInfo>)>>,
    remote_out: mpsc::Sender<graphics::Result>,
) -> IcmtResult<()> {
    /* Create our own agent here since we cannot Send it here from the main thread. */
    let canister_id = Principal::from_text(cfg.canister_id.clone()).unwrap();
    let agent = create_agent(&cfg.replica_url)?;
    let ctx = ConnectCtx {
        cfg: cfg.clone(),
        canister_id,
        agent,
    };

    loop {
        let events = remote_in.recv()?;

        match events {
            None => return Ok(()),
            Some((window_dim, events)) => {
                let mut rr = service_call(&ctx, ServiceCall::View(window_dim, events)).await?;
                assert_eq!(rr.len(), 1);
                remote_out.send(rr.remove(0))?;
            }
        }
    }
}

async fn do_update_task(
    cfg: ConnectCfg,
    remote_in: mpsc::Receiver<ServiceCall>,
    remote_out: mpsc::Sender<()>,
) -> IcmtResult<()> {
    /* Create our own agent here since we cannot Send it here from the main thread. */
    let canister_id = Principal::from_text(cfg.canister_id.clone()).unwrap();
    let agent = create_agent(&cfg.replica_url)?;
    let ctx = ConnectCtx {
        cfg,
        canister_id,
        agent,
    };
    loop {
        let sc = remote_in.recv().unwrap();
        if let ServiceCall::FlushQuit = sc {
            return Ok(());
        };
        service_call(&ctx, sc).await?;
        remote_out.send(()).unwrap();
    }
}

async fn local_event_loop(ctx: ConnectCtx) -> Result<(), IcmtError> {
    let mut window_dim = graphics::Dim {
        width: Nat::from(320),
        height: Nat::from(240),
    }; // use CLI to init

    let sdl = sdl2::init()?;

    // to do --- if headless, do not do these steps; window_canvas is None
    let video_subsystem = sdl.video()?;
    let window = video_subsystem
        .window(
            "IC Mini Terminal",
            nat_ceil(&window_dim.width),
            nat_ceil(&window_dim.height),
        )
        .position_centered()
        .resizable()
        /*.input_grabbed() // to do -- CI flag*/
        .build()
        .map_err(|e| e.to_string())?;

    let mut window_canvas = window
        .into_canvas()
        .target_texture()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // to do --- if file-less, do not do these steps; file_canvas is None
    let mut file_canvas = {
        let surface = sdl2::surface::Surface::new(
            nat_ceil(&window_dim.width),
            nat_ceil(&window_dim.height),
            sdl2::pixels::PixelFormatEnum::RGBA8888,
        )?;
        surface.into_canvas()?
    };

    let mut view_events = vec![];
    let mut update_events = vec![];

    let mut dump_events = vec![];
    let mut engiffen_paths = vec![];

    let (update_in, update_out) = /* Begin update task */ {
        let cfg = ctx.cfg.clone();

        // Interaction cycle as two halves (local/remote); each half is a thread.
        // There are four end points along the cycle's halves:
        let (local_out, remote_in) = mpsc::channel::<ServiceCall>();
        let (remote_out, local_in) = mpsc::channel::<()>();

        // 1. Remote interactions via update calls to service.
        // (Consumes remote_in and produces remote_out).

        task::spawn(do_update_task(cfg, remote_in, remote_out));
        local_out.send(ServiceCall::Update(vec![skip_event(&ctx)], graphics::Request::None))?;
        (local_in, local_out)
    };

    let (view_in, view_out) = /* Begin view task */ {
        let cfg = ctx.cfg.clone();

        // Interaction cycle as two halves (local/remote); each half is a thread.
        // There are four end points along the cycle's halves:
        let (local_out, remote_in) = mpsc::channel::<Option<(graphics::Dim, Vec<event::EventInfo>)>>();
        let (remote_out, local_in) = mpsc::channel::<graphics::Result>();

        // 1. Remote interactions via view calls to service.
        // (Consumes remote_in and produces remote_out).

        task::spawn(do_view_task(cfg, remote_in, remote_out));
        local_out.send(Some((window_dim.clone(), vec![skip_event(&ctx)])))?;
        (local_in, local_out)
    };

    let mut quit_request = false; // user has requested to quit: shut down gracefully.
    let mut dirty_flag = true; // more events ready for view task
    let mut ready_flag = true; // view task is ready for more events

    let mut update_requests = Nat::from(1); // count update task requests (already one).
    let mut update_responses = Nat::from(0); // count update task responses (none yet).

    let mut view_requests = Nat::from(1); // count view task requests (already one).
    let mut view_responses = Nat::from(0); // count view task responses (none yet).

    // 2. Local interactions via the SDL Event loop.
    let mut event_pump = {
        use sdl2::event::EventType;
        let mut p = sdl.event_pump()?;
        p.disable_event(EventType::FingerUp);
        p.disable_event(EventType::FingerDown);
        p.disable_event(EventType::FingerMotion);
        p.disable_event(EventType::MouseMotion);
        p
    };

    'running: loop {
        if let Some(system_event) = event_pump.wait_event_timeout(13) {
            // utc/local timestamps for event
            let event = translate_system_event(&video_subsystem, &system_event);
            let event = match event {
                None => continue 'running,
                Some(event) => event,
            };
            trace!("SDL event_pump.wait_event() => {:?}", &system_event);
            // catch window resize event: redraw and loop:
            match event {
                event::Event::MouseDown(_) => {
                    // ignore (for now)
                }
                event::Event::Skip => {
                    // ignore
                }
                event::Event::Quit => {
                    info!("Quit");
                    println!("Begin: Quitting...");
                    println!("Waiting for next update response...");
                    quit_request = true;
                }
                event::Event::ClipBoard(text) => {
                    info!("ClipBoard: {}", text);
                    dirty_flag = true;
                    let ev = event::EventInfo {
                        user_info: event::UserInfo {
                            user_name: ctx.cfg.user_info.0.clone(),
                            text_color: (
                                ctx.cfg.user_info.1.clone(),
                                (Nat::from(0), Nat::from(0), Nat::from(0)),
                            ),
                        },
                        nonce: None,
                        date_time_local: Local::now().to_rfc3339(),
                        date_time_utc: Utc::now().to_rfc3339(),
                        event: event::Event::ClipBoard(text),
                    };
                    view_events.push(ev.clone());
                    dump_events.push(ev);
                }
                event::Event::WindowSize(new_dim) => {
                    info!("WindowSize {:?}", new_dim);
                    dirty_flag = true;
                    view_events.push(skip_event(&ctx));
                    dump_events.push(skip_event(&ctx));
                    write_gifs(&ctx.cfg.cli_opt, &window_dim, dump_events, &engiffen_paths)?;
                    dump_events = vec![];
                    engiffen_paths = vec![];
                    window_dim = new_dim;
                    // to do -- add event to buffer, and send to service
                    file_canvas = {
                        // Re-size canvas by re-creating it.
                        let surface = sdl2::surface::Surface::new(
                            nat_ceil(&window_dim.width),
                            nat_ceil(&window_dim.height),
                            sdl2::pixels::PixelFormatEnum::RGBA8888,
                        )?;
                        surface.into_canvas()?
                    };
                }
                event::Event::KeyDown(ref keys) => {
                    info!("KeyDown {:?}", keys);
                    dirty_flag = true;
                    let ev = event::EventInfo {
                        user_info: event::UserInfo {
                            user_name: ctx.cfg.user_info.0.clone(),
                            text_color: (
                                ctx.cfg.user_info.1.clone(),
                                (Nat::from(0), Nat::from(0), Nat::from(0)),
                            ),
                        },
                        nonce: None,
                        date_time_local: Local::now().to_rfc3339(),
                        date_time_utc: Utc::now().to_rfc3339(),
                        event: event::Event::KeyDown(keys.clone()),
                    };
                    view_events.push(ev.clone());
                    dump_events.push(ev);
                }
            }
        };

        /* attend to update task */
        {
            match update_in.try_recv() {
                Ok(()) => {
                    update_responses += 1;
                    info!("update_responses = {}", update_responses);
                    update_out
                        .send(ServiceCall::Update(
                            view_events.clone(),
                            graphics::Request::All(window_dim.clone()),
                        ))
                        .unwrap();
                    if quit_request {
                        println!("Continue: Quitting...");
                        println!("Waiting for final update-task response.");
                        match update_in.try_recv() {
                            Ok(()) => {
                                update_out.send(ServiceCall::FlushQuit)?;
                                println!("Done.");
                            }
                            Err(e) => return Err(IcmtError::String(e.to_string())),
                        }
                    };
                    update_requests += 1;
                    info!("update_requests = {}", update_requests);
                    update_events = view_events;
                    view_events = vec![];
                    dirty_flag = true;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    if quit_request {
                        println!("Continue: Quitting...");
                        println!("Waiting for final update-task response.");
                        update_in.recv()?;
                        update_out.send(ServiceCall::FlushQuit)?;
                        println!("Done.");
                    } else {
                        /* not ready; do nothing */
                    }
                }
                Err(e) => {
                    error!("Update task error: {:?}", e);
                    println!("Cannot recover; quiting...");
                    quit_request = true;
                }
            }
        };

        if quit_request {
            write_gifs(&ctx.cfg.cli_opt, &window_dim, dump_events, &engiffen_paths)?;
            {
                print!("Stopping view task... ");
                view_out.send(None)?;
                println!("Done.");
            }
            println!("All done.");
            return Ok(());
        } else
        /* attend to view task */
        {
            match view_in.try_recv() {
                Ok(rr) => {
                    view_responses += 1;
                    info!("view_responses = {}", view_responses);

                    do_redraw(
                        &(ctx.cfg).cli_opt,
                        &window_dim,
                        &mut window_canvas,
                        &mut file_canvas,
                        &mut engiffen_paths,
                        &rr,
                    )
                    .await?;

                    ready_flag = true;
                }
                Err(mpsc::TryRecvError::Empty) => { /* not ready; do nothing */ }
                Err(e) => error!("{:?}", e),
            };

            if dirty_flag && ready_flag {
                dirty_flag = false;
                ready_flag = false;
                let mut events = update_events.clone();
                events.append(&mut (view_events.clone()));

                view_out.send(Some((window_dim.clone(), events)))?;

                view_requests += 1;
                info!("view_requests = {}", view_requests);
            }
        };

        // attend to next batch of local events, and loop everything above
        continue 'running;
    }
}

// to do -- fix hack; refactor to remove Option<_> in return type
pub async fn service_call(
    ctx: &ConnectCtx,
    call: ServiceCall,
) -> IcmtResult<Vec<graphics::Result>> {
    if let ServiceCall::FlushQuit = call {
        return Ok(vec![]);
    };
    debug!(
        "service_call: to canister_id {:?} at replica_url {:?}",
        ctx.cfg.canister_id, ctx.cfg.replica_url
    );
    let delay = Delay::builder()
        .throttle(RETRY_PAUSE)
        .timeout(REQUEST_TIMEOUT)
        .build();
    let timestamp = std::time::SystemTime::now();
    info!("service_call: {:?}", call);
    let arg_bytes = match call.clone() {
        ServiceCall::FlushQuit => candid::encode_args(()).unwrap(),
        ServiceCall::View(window_dim, evs) => candid::encode_args((window_dim, evs)).unwrap(),
        ServiceCall::Update(evs, req) => candid::encode_args((evs, req)).unwrap(),
    };
    info!(
        "service_call: Encoded argument via Candid; Arg size {:?} bytes",
        arg_bytes.len()
    );
    info!("service_call: Awaiting response from service...");
    // do an update or query call, based on the ServiceCall case:
    let blob_res = match call.clone() {
        ServiceCall::FlushQuit => None,
        ServiceCall::View(_window_dim, _keys) => {
            let resp = ctx
                .agent
                .query(&ctx.canister_id, "view")
                .with_arg(arg_bytes)
                .call()
                .await?;
            Some(resp)
        }
        ServiceCall::Update(_keys, _req) => {
            let resp = ctx
                .agent
                .update(&ctx.canister_id, "update")
                .with_arg(arg_bytes)
                .call_and_wait(delay)
                .await?;
            Some(resp)
        }
    };
    let elapsed = timestamp.elapsed().unwrap();
    if let Some(blob_res) = blob_res {
        info!(
            "service_call: Ok: Response size {:?} bytes; elapsed time {:?}",
            blob_res.len(),
            elapsed
        );
        match call.clone() {
            ServiceCall::FlushQuit => Ok(vec![]),
            ServiceCall::Update(_, _) => match candid::Decode!(&(*blob_res), Vec<graphics::Result>)
            {
                Ok(res) => Ok(res),
                Err(candid_err) => {
                    error!("Candid decoding error: {:?}", candid_err);
                    Err(IcmtError::String("decoding error".to_string()))
                }
            },
            ServiceCall::View(_, _) => match candid::Decode!(&(*blob_res), graphics::Result) {
                Ok(res) => {
                    if ctx.cfg.cli_opt.log_trace {
                        let mut res_log = format!("{:?}", &res);
                        if res_log.len() > 1000 {
                            res_log.truncate(1000);
                            res_log.push_str("...(truncated)");
                        }
                        trace!(
                            "service_call: Successful decoding of graphics output: {:?}",
                            res_log
                        );
                    }
                    Ok(vec![res])
                }
                Err(candid_err) => {
                    error!("Candid decoding error: {:?}", candid_err);
                    Err(IcmtError::String("decoding error".to_string()))
                }
            },
        }
    } else {
        error!(
            "service_call: Error result: {:?}; elapsed time {:?}", blob_res, elapsed
        );
        Err(IcmtError::String("ic-mt error".to_string()))
    }
}

fn main() -> IcmtResult<()> {
    use tokio::runtime::Runtime;
    let mut runtime = Runtime::new().expect("Unable to create a runtime");

    let cli_opt = CliOpt::from_args();
    init_log(
        match (cli_opt.log_trace, cli_opt.log_debug, cli_opt.log_info) {
            (true, _, _) => log::LevelFilter::Trace,
            (_, true, _) => log::LevelFilter::Debug,
            (_, _, true) => log::LevelFilter::Info,
            (_, _, _) => log::LevelFilter::Warn,
        },
    );
    info!("Evaluating CLI command: {:?} ...", &cli_opt.command);
    let c = cli_opt.command.clone();
    match c {
        CliCommand::Completions { shell: s } => {
            // see also: https://clap.rs/effortless-auto-completion/
            CliOpt::clap().gen_completions_to("icmt", s, &mut io::stdout());
            info!("done")
        }
        CliCommand::Connect {
            canister_id,
            replica_url,
            user_info_text,
        } => {
            let capout = std::path::Path::new(&cli_opt.capture_output_path);
            if !capout.exists() {
                std::fs::create_dir_all(&cli_opt.capture_output_path)?;
            };
            let raw_args: (String, (u8, u8, u8)) = ron::de::from_str(&user_info_text).unwrap();
            let user_info: UserInfoCli = {
                (
                    raw_args.0,
                    (
                        Nat::from((raw_args.1).0),
                        Nat::from((raw_args.1).1),
                        Nat::from((raw_args.1).2),
                    ),
                )
            };
            let cfg = ConnectCfg {
                canister_id,
                replica_url,
                cli_opt,
                user_info,
            };
            let canister_id = Principal::from_text(cfg.canister_id.clone()).unwrap();
            let agent = create_agent(&cfg.replica_url)?;
            let ctx = ConnectCtx {
                cfg,
                canister_id,
                agent,
            };
            info!("Connecting to IC canister: {:?}", ctx.cfg);
            runtime.block_on(local_event_loop(ctx)).ok();
        }
    };
    Ok(())
}
