#!/usr/bin/env python3
"""Render the real flight simulation, then mix the supplied theme and own effects."""
from pathlib import Path
import json
import os
import shutil
import subprocess

root = Path(__file__).resolve().parents[1]
ffmpeg = shutil.which("ffmpeg")
if not ffmpeg:
    try:
        import imageio_ffmpeg
        ffmpeg = imageio_ffmpeg.get_ffmpeg_exe()
    except ImportError as error:
        raise SystemExit("Para regenerar el video instala FFmpeg o imageio-ffmpeg. El juego no lo necesita.") from error
env = dict(os.environ, COSMOS_FFMPEG=ffmpeg)
subprocess.run(["cargo", "run", "--release", "--locked", "--example", "flight_demo"], cwd=root, env=env, check=True)
output = root / "deliverable"
duration = json.loads((output / "video-evidencia.json").read_text())["duration_seconds"]
music = root / "assets/audio/local.mp3"
args = [ffmpeg, "-hide_banner", "-loglevel", "warning", "-y", "-i", str(output / "vuelo-sin-audio.mp4")]
if music.is_file():
    args += ["-i", str(music), "-i", str(output / "propulsores.wav"), "-filter_complex",
             f"[1:a]volume=0.28,afade=t=in:d=2,afade=t=out:st={duration-4}:d=4[music];[2:a]volume=0.55[engine];[music][engine]amix=inputs=2:duration=shortest:normalize=0[a]",
             "-map", "0:v", "-map", "[a]"]
else:
    args += ["-i", str(output / "propulsores.wav"), "-map", "0:v", "-map", "1:a"]
args += ["-c:v", "copy", "-c:a", "aac", "-b:a", "192k", "-t", str(duration), "-movflags", "+faststart", str(output / "COSMOS_GP_DEMO.mp4")]
subprocess.run(args, check=True)
print(output / "COSMOS_GP_DEMO.mp4")
