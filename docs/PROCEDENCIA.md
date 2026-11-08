# Proyecto 3: Space Travel — procedencia y contribuciones

## Autoría y asistencia

Trabajo individual de Yehosua F. Hercules. Se utilizó Codex para acordar alcance,
diseñar módulos, escribir y modificar código, depurar, producir pruebas, medir
rendimiento, preparar documentación y generar el video reproducible. El código
nuevo no debe presentarse como escrito sin asistencia de IA.

Las decisiones de experiencia fueron acordadas con el usuario: identidad espacial
inspirada en F1, planetas existentes, Nave Aster R, inercia y asistencia orbital,
cámaras, puerto de competición, contrarreloj y uso de una pista local de F1.
La comprensión técnica y la explicación individual deben validarse en el ensayo.

## Recursos previos del proyecto

- `Nave_Espacial/Nave_Aster.obj` y MTL: modelo previo de Nave Aster R. Se reutiliza
  el archivo triangulado de 557 posiciones y 1,110 triángulos, con cinco materiales.
  El cargador adapta su eje frontal al convenio -Z del vuelo.
- `lab5-shaders`: base del ruido continuo, identidades de los cuerpos, shaders y
  tipografía de píxeles. La proyección de aquel laboratorio era ortográfica;
  el vuelo con perspectiva de este proyecto es una evolución nueva.
- `lab-4`: referencia del lector propio de OBJ/MTL.

Los modelos de superficie, sus distribuciones y los aros se generan en el código
incluido. No se descargaron modelos 3D ni texturas planetarias. El cielo, el mapa de
normales y los campos de material también se calculan proceduralmente.

## Bibliotecas y herramientas

| Recurso | Versión / uso |
| --- | --- |
| Rust y biblioteca estándar | Simulación, estructuras de datos, archivos y paralelismo. |
| glam | 0.27.0; vectores, matrices y cuaterniones. La cámara y fórmula de proyección se implementan en el proyecto. |
| image | 0.24.9; escritura de capturas PNG. |
| raylib-rs / raylib | Dependencias 6.0.0; ventana, entrada, audio y presentación del framebuffer como textura 2D. |
| FFmpeg | Generación y mezcla del MP4; no se necesita para jugar. |
| Python | Scripts de empaquetado y orquestación del video. |

Las versiones transitivas están en Cargo.lock. Se adjuntan los avisos de las
bibliotecas directas disponibles en las dependencias locales, además de raylib y GLFW.

## Música

Composición: **Formula 1 Theme**, Brian Tyler. Referencia oficial consultada:
[Formula 1 Theme Live in Concert by Brian Tyler](https://www.youtube.com/watch?v=8AYy-BcjRXg).

El usuario proporcionó el archivo local
`formula-1-theme-live-in-concert-by-brian-tyler-128-ytshorts.savetube.me.mp3`.
Se verificó un MP3 estéreo de 48 kHz, 128 kb/s y aproximadamente 3:27.
La copia de trabajo se llama `assets/audio/local.mp3`, se incluye en la aplicación
local y está excluida de Git. No se atribuye la grabación ni la composición al autor
del proyecto. El enlace oficial permanece disponible con F8.

Los propulsores y señales breves de carrera se sintetizan en `audio.rs`; no usan
muestras de motores descargadas.

## Video

`examples/flight_demo.rs` produce una captura determinista: un controlador emite
las mismas entradas de aceleración, giro, freno y altura que recibe la simulación
interactiva. Después de inicializar la nave no asigna directamente su posición ni
su velocidad. Usa la cámara del juego y registra la trayectoria.

El video se presenta como demostración reproducible del motor; no es una grabación
de una vuelta manual ni evidencia de que la ventana ejecuta a la frecuencia del MP4.
La interacción con teclado, cámara, carrera y volumen se comprobó por separado
en la aplicación nativa.
