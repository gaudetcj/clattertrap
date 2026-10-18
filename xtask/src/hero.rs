//! `cargo xtask hero`: records the arena grid in the browser and assembles `docs/hero.gif` and
//! `docs/hero.mp4` with ffmpeg. The browser side writes `frames/NNNNN.png` for the whole run,
//! `gif/NNNNN.png` for the shorter GIF, and `audio.wav`.

use crate::{step_env, wasm};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Frames per second the recorder writes and the MP4 plays at.
pub const FPS: u32 = 30;

/// Frames per second of the GIF, a subset of the recorded frames.
const GIF_FPS: u32 = 12;

/// The environment the recorder reads: the level, the arm, and where to write.
#[must_use]
pub fn hero_env(level: &str, arm: &str, out: &Path) -> Vec<(String, String)> {
    vec![
        ("HERO_LEVEL".to_owned(), level.to_owned()),
        ("HERO_ARM".to_owned(), arm.to_owned()),
        ("HERO_OUT".to_owned(), out.display().to_string()),
    ]
}

fn frames_pattern(out: &Path, set: &str) -> String {
    out.join(set).join("%05d.png").display().to_string()
}

/// ffmpeg arguments for the MP4: H.264 in yuv420p so every player opens it, with the rendered audio.
#[must_use]
pub fn mp4_args(out: &Path, wav: &Path, mp4: &Path) -> Vec<String> {
    [
        "-y",
        "-framerate",
        &FPS.to_string(),
        "-i",
        &frames_pattern(out, "frames"),
        "-i",
        &wav.display().to_string(),
        "-c:v",
        "libx264",
        "-pix_fmt",
        "yuv420p",
        "-crf",
        "18",
        "-preset",
        "slow",
        "-tune",
        "animation",
        "-c:a",
        "aac",
        "-b:a",
        "128k",
        "-shortest",
        "-movflags",
        "+faststart",
        &mp4.display().to_string(),
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

/// ffmpeg arguments for the GIF: the shorter frame set, a palette made from it, no dither, looping.
#[must_use]
pub fn gif_args(out: &Path, gif: &Path) -> Vec<String> {
    let filter = format!(
        "fps={GIF_FPS},split[a][b];[a]palettegen=max_colors=64:stats_mode=diff[p];[b][p]paletteuse=dither=none"
    );
    [
        "-y",
        "-framerate",
        &FPS.to_string(),
        "-i",
        &frames_pattern(out, "gif"),
        "-vf",
        &filter,
        "-loop",
        "0",
        &gif.display().to_string(),
    ]
    .iter()
    .map(ToString::to_string)
    .collect()
}

fn as_str(args: &[String]) -> Vec<&str> {
    args.iter().map(String::as_str).collect()
}

fn require_ffmpeg() -> Result<()> {
    let status = Command::new("ffmpeg")
        .arg("-version")
        .output()
        .context("ffmpeg is not installed; hero needs ffmpeg on PATH (winget install Gyan.FFmpeg, or apt install ffmpeg)")?
        .status;
    anyhow::ensure!(status.success(), "ffmpeg -version failed with {status}");
    Ok(())
}

/// Records `level` under `arm` and writes `docs/hero.gif` and `docs/hero.mp4`.
///
/// # Errors
/// When ffmpeg is missing, the wasm build fails, the browser recorder fails, or ffmpeg fails.
pub fn record(root: &Path, level: &str, arm: &str) -> Result<()> {
    require_ffmpeg()?;
    wasm::build(root)?;
    let out: PathBuf = root.join("target").join("hero");
    if out.exists() {
        std::fs::remove_dir_all(&out).with_context(|| format!("clearing {}", out.display()))?;
    }
    std::fs::create_dir_all(out.join("frames"))
        .with_context(|| format!("creating {}", out.display()))?;
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };
    step_env(
        root,
        npm,
        &["--prefix", "web", "run", "hero"],
        &hero_env(level, arm, &out),
    )
    .context("the browser recorder failed; if it reports a missing browser, run: npm --prefix web run playwright:install")?;
    let docs = root.join("docs");
    let mp4 = mp4_args(&out, &out.join("audio.wav"), &docs.join("hero.mp4"));
    let gif = gif_args(&out, &docs.join("hero.gif"));
    step_env(root, "ffmpeg", &as_str(&mp4), &[])?;
    step_env(root, "ffmpeg", &as_str(&gif), &[])?;
    for name in ["hero.mp4", "hero.gif"] {
        let path = docs.join(name);
        let kb = std::fs::metadata(&path).map_or(0, |m| m.len() / 1024);
        println!("wrote {} ({kb} KB)", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FPS, gif_args, hero_env, mp4_args};
    use std::path::Path;

    #[test]
    fn the_recorder_gets_the_level_arm_and_output_directory_in_its_environment() {
        let env = hero_env("seesaw-launch", "blueprint", Path::new("target/hero"));
        assert_eq!(
            env,
            [
                ("HERO_LEVEL".to_owned(), "seesaw-launch".to_owned()),
                ("HERO_ARM".to_owned(), "blueprint".to_owned()),
                (
                    "HERO_OUT".to_owned(),
                    Path::new("target/hero").display().to_string()
                ),
            ]
        );
    }

    #[test]
    fn the_mp4_is_h264_yuv420p_with_the_rendered_audio() {
        let args = mp4_args(
            Path::new("target/hero"),
            Path::new("target/hero/audio.wav"),
            Path::new("docs/hero.mp4"),
        );
        let text = args.join(" ");
        assert!(text.contains(&format!("-framerate {FPS}")), "{text}");
        assert!(text.contains("-c:v libx264"), "{text}");
        assert!(text.contains("-pix_fmt yuv420p"), "{text}");
        assert!(text.contains("audio.wav"), "{text}");
        assert!(text.contains("-c:a aac"), "{text}");
        assert!(text.ends_with("hero.mp4"), "{text}");
        assert!(
            args.iter()
                .any(|a| a.ends_with("%05d.png") && a.contains("frames")),
            "{text}"
        );
    }

    #[test]
    fn the_gif_is_built_through_a_generated_palette_without_dither() {
        let args = gif_args(Path::new("target/hero"), Path::new("docs/hero.gif"));
        let text = args.join(" ");
        assert!(text.contains("palettegen"), "{text}");
        assert!(text.contains("paletteuse=dither=none"), "{text}");
        assert!(text.contains("fps=12"), "{text}");
        assert!(text.contains("-loop 0"), "{text}");
        assert!(!text.contains("audio"), "{text}");
        assert!(
            args.iter()
                .any(|a| a.ends_with("%05d.png") && a.contains("gif")),
            "{text}"
        );
    }
}
