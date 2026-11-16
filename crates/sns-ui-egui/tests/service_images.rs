//! Qualify all four image formats against real HTTP bytes and the native decoder.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use image::{ImageFormat, Rgb, RgbImage};
use sns_engine::{Engine, Limits};
use sns_model::*;
#[test]
fn network_browser_loads_png_bmp_gif_jpeg_from_simulated_http() {
    let mut p: sns_model::Project =
        serde_json::from_str(include_str!("../../../fixtures/valid/services.json")).unwrap();
    let mut html = String::from("<h1>Formats</h1>");
    for (ext, format) in [
        ("png", ImageFormat::Png),
        ("bmp", ImageFormat::Bmp),
        ("gif", ImageFormat::Gif),
        ("jpeg", ImageFormat::Jpeg),
    ] {
        let mut buffer = std::io::Cursor::new(vec![]);
        RgbImage::from_pixel(12, 8, Rgb([20, 150, 200]))
            .write_to(&mut buffer, format)
            .unwrap();
        let data = buffer.into_inner();
        html += &format!("<img src=\"/image.{ext}\" alt=\"{ext}\">");
        p.devices[1].files.push(VirtualFile {
            path: format!("/www/image.{ext}"),
            media_type: format!("image/{ext}"),
            data_base64: STANDARD.encode(data),
        });
    }
    p.devices[1]
        .files
        .iter_mut()
        .find(|f| f.path == "/www/index.html")
        .unwrap()
        .data_base64 = STANDARD.encode(html);
    let mut e = Engine::from_project(p, 42, Limits::default()).unwrap();
    let id = |s: &str| Id::new(s).unwrap();
    let commands = [
        (
            "host-b",
            AppCommand::StartServer {
                application: id("web"),
                port: 80,
            },
        ),
        (
            "host-a",
            AppCommand::Navigate {
                application: id("host-a-browser"),
                url: "http://192.168.1.3/".into(),
                fields: None,
            },
        ),
    ];
    for (index, (host, command)) in commands.into_iter().enumerate() {
        if index == 1 {
            e.run(10000, Some(1_000_000)).unwrap();
        }
        e.command(CommandEnvelope {
            api_version: 1,
            command_id: id(&format!("formats-{index}")),
            project_id: e.project().project_id.clone(),
            run_id: e.run_id().clone(),
            expected_revision: Decimal(e.revision()),
            command: Command::App {
                device: id(host),
                command,
            },
        })
        .unwrap();
    }
    e.run(10000, Some(3_000_000)).unwrap();
    let browser = e
        .apps()
        .views(e.project())
        .into_iter()
        .find(|h| h.device == id("host-a"))
        .unwrap()
        .applications
        .into_iter()
        .find(|a| a.software == "web_browser")
        .unwrap()
        .browser;
    assert_eq!(browser.images.len(), 4);
    for image in browser.images.values() {
        let bytes = STANDARD.decode(&image.data_base64).unwrap();
        let decoded = image::load_from_memory(&bytes).unwrap();
        assert_eq!((decoded.width(), decoded.height()), (12, 8));
    }
    assert_eq!(
        e.trace()
            .iter()
            .filter(|f| f.kind == "http.response.received" && f.payload["resource"] == true)
            .count(),
        4
    );
}
