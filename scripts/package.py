#!/usr/bin/env python3
"""Create the final local hand-in, excluding build caches and intermediate media."""
from pathlib import Path
import hashlib
import json
import subprocess
import sys
import zipfile

root = Path(__file__).resolve().parents[1]
output = root / "deliverable"
video = output / "COSMOS_GP_DEMO.mp4"
evidence = output / "video-evidencia.json"
if not video.is_file() or not evidence.is_file():
    raise SystemExit("Primero generar el video con scripts/render_video.py")
data = json.loads(evidence.read_text())
if not (60 <= data["duration_seconds"] <= 90 and data["orbit_degrees"] >= 360 and data["impacts"] == 0):
    raise SystemExit("La evidencia del recorrido no cumple la validación de entrega")
subprocess.run([sys.executable, str(root / "scripts/build_macos.py")], cwd=root, check=True)
archive = output / "PROYECTO_3_SPACE_TRAVEL.zip"
paths = [root / name for name in ("Cargo.toml", "Cargo.lock", ".gitignore", "README.md")]
for folder in ("src", "assets", "tests", "examples", "scripts", "docs", "licenses", "COSMOS GP.app"):
    for path in sorted((root / folder).rglob("*")):
        if path.is_file() and path.name not in (".DS_Store", "ACUERDOS.md") and "__pycache__" not in path.parts:
            paths.append(path)
manifest = []
with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as bundle:
    for path in paths:
        relative = path.relative_to(root).as_posix()
        bundle.write(path, "COSMOS_GP/" + relative)
        manifest.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {relative}")
    for name in ("COSMOS_GP_DEMO.mp4", "video-evidencia.json", "trayectoria.csv"):
        path = output / name
        bundle.write(path, "COSMOS_GP/demostracion/" + name)
        manifest.append(f"{hashlib.sha256(path.read_bytes()).hexdigest()}  demostracion/{name}")
    bundle.writestr("COSMOS_GP/MANIFIESTO_SHA256.txt", "\n".join(manifest) + "\n")
    bundle.writestr("COSMOS_GP/EMPIEZA_AQUI.txt", """PROYECTO 3: SPACE TRAVEL
COSMOS / GP — ASTER R

1. Abrir COSMOS GP.app en un Mac con Apple Silicon.
2. W acelera, flechas orientan, Espacio frena. H muestra todos los controles.
3. N prepara el contrarreloj. P pausa. M silencia el audio.
4. Ver demostracion/COSMOS_GP_DEMO.mp4: recorrido continuo de aproximadamente un minuto.
5. README.md explica compilación, controles y herramientas.
6. docs/CRITERIOS.md corresponde al menú de evaluación del PDF.
7. docs/GUIA_DEFENSA.md prepara la explicación y los cambios de parámetros.
8. docs/PROCEDENCIA.md declara recursos, música y asistencia de IA.

El código fuente y los recursos están incluidos; target y medios intermedios no.
El video usa entradas reproducibles a la misma física del juego.
""")
with zipfile.ZipFile(archive) as bundle:
    damaged = bundle.testzip()
    if damaged:
        raise SystemExit(f"Archivo dañado: {damaged}")
digest = hashlib.sha256(archive.read_bytes()).hexdigest()
archive.with_suffix(".zip.sha256").write_text(f"{digest}  {archive.name}\n")
print(f"{archive}\n{archive.stat().st_size / 1024 / 1024:.2f} MiB\nSHA256 {digest}")
