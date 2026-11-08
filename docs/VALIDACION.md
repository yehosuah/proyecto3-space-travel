# Proyecto 3: Space Travel — registro de validación

7 de noviembre de 2026, Mac del usuario (Apple M5 Pro, macOS).

## Versión de entrega

- `cargo test --release --locked --offline`: **18 pruebas aprobadas**, sin fallos.
- `cargo clippy --all-targets --locked --offline -- -D warnings`: aprobado.
- `cargo fmt --check`: aprobado.
- Se añadió una regresión para el seguimiento de cámara durante retropropulsión:
  conserva la separación respecto de la nave al trasladarse hacia atrás.
- El video final tiene **67.067 segundos**, 2,012 cuadros, H.264 a 1440 × 900 y
  30 FPS; audio AAC estéreo a 48 kHz. Se decodificó completo con FFmpeg sin errores.
- La trayectoria recorre **362.35°** alrededor de Mónaco y registra **cero impactos**.
  Altitud mínima respecto del radio base: 35.37 unidades; final: 675.03 unidades.
  Mayor desplazamiento entre cuadros de captura: 2.534 unidades.
- Se revisaron encuadres del acercamiento, dos puntos de la vuelta, la salida y el
  cierre. La señal de audio mezclada alcanzó un pico de −11.4 dBFS, sin saturación;
  esta medición no equivale a una evaluación auditiva subjetiva.

`video-evidencia.json` y `trayectoria.csv` acompañan al MP4 en `demostracion/`.
El video se genera con entradas programadas a la misma simulación del juego;
no se presenta como pilotaje manual ni como benchmark de la ventana.

### Entrega reproducible

- Se extrajo el ZIP en una carpeta temporal y se verificaron los SHA-256 de todos
  los archivos incluidos en su manifiesto, además del CRC del archivo comprimido.
- La aplicación extraída generó la vista `puerto` con el directorio de trabajo en
  `/private/tmp`. En un paquete `.app`, la búsqueda de modelos exige sus propios
  recursos; no puede recurrir a los archivos de la carpeta de desarrollo.
- Se abrió esa copia en una ventana nativa y se probaron retropropulsión, freno,
  pausa, diagnóstico F3 y captura F12. La cámara conservó la vista externa de la
  nave. La captura se escribió junto a la copia extraída y mostró la pista F1 cargada.
- Se observaron 58.4 FPS en esa comprobación a 960 × 600 internos. Es una lectura
  puntual del indicador suavizado, no un mínimo garantizado.
- El código extraído se compiló en un directorio de construcción independiente
  y volvió a aprobar las 18 pruebas con `cargo test --release --locked --offline`.
  El modo offline utilizó las dependencias ya instaladas en este Mac.
- `otool -L` mostró únicamente bibliotecas del sistema. OpenGL se usa mediante
  Raylib para presentar la textura 2D que contiene el framebuffer calculado en CPU.

El ZIP contiene fuente, recursos, aplicación para Apple Silicon, instrucciones,
documentación y demostración. No contiene `target` ni medios intermedios.
No se probó otro sistema operativo ni otra arquitectura de Mac.

## Historial de la etapa 1

- `cargo test --release --locked --offline`: 9 pruebas aprobadas.
- `cargo clippy --all-targets --locked --offline -- -D warnings`: aprobado.
- `cargo fmt --check`: aprobado.
- El paquete `.app` abrió mediante la interfaz nativa; se verificaron pausa visible,
  instrumentación y guardado F12 junto a la aplicación. La resolución interna es 960 × 600.
- Se probaron aceleración, frenado, giro y cambio de cámara mediante teclado en la
  ventana. El control orbital y su seguimiento del planeta tienen prueba automatizada.
- El ejecutable empaquetado también produjo una captura con el directorio de trabajo
  en `/private/tmp`, sin depender del directorio desde donde se abrió.

## Rendimiento observado

Comparación antes/después del reparto del rasterizado entre hilos, 960 × 600.
La versión paralela se midió durante 80 cuadros por escena, descartando los 3 primeros.
Son mediciones del renderizado CPU; no incluyen presentación de ventana ni captura de pantalla.

| Escena | Media anterior, ms | Media paralela, ms | P95 paralelo, ms |
| --- | ---: | ---: | ---: |
| Inicio | 12.89 | 6.90 | 7.16 |
| Superficie | 44.73 | 9.91 | 10.37 |
| Anillos | 28.55 | 6.40 | 6.55 |

En la aplicación nativa se observaron aproximadamente 57 FPS en el indicador
suavizado (56.9 en la captura final). Esto no es una garantía de mínimo sostenido
en todo el recorrido. La comparación de píxeles y profundidad entre rasterización
secuencial y paralela pasa con igualdad exacta en la escena de prueba.

## Cobertura de las pruebas

1. Simulación equivalente con ritmos de renderizado de 30 y 144 FPS.
2. El impulso no atraviesa un planeta entre pasos de simulación.
3. Frenado y conservación de velocidad sin asistencia ni gravedad.
4. Modo orbital alrededor de un planeta que se mueve.
5. Interpolación perspectiva sin depender de la diagonal de un cuadrilátero.
6. Clipping de geometría que cruza el plano cercano.
7. Profundidad y descarte de fragmentos independientes del orden de dibujo.
8. Frustum culling conserva esferas que intersectan el volumen visible.
9. Rasterización paralela conserva píxeles y profundidad respecto de la secuencial.

### Pendientes identificados al terminar la etapa 1

Revisión del pilotaje por el usuario, detalle visual y modelos de superficie,
contrarreloj, audio, evidencia comparativa de optimizaciones en la versión final,
video de vuelo continuo y explicación técnica con atribuciones.
Estos puntos se atendieron en las etapas posteriores descritas en este registro.
La revisión y el ensayo personal del usuario siguen siendo parte de la preparación
para defenderlo; la documentación no representa una nota obtenida.

## Etapa 2

Las 15 pruebas pasan, incluidas distribución reproducible y orientación de modelos,
cruce dirigido y secuencial de puertas, vuelta completa y persistencia del récord,
mapa de normales no plano y señal PCM acotada que responde al impulso. Las pruebas
de la carrera usan trayectorias controladas; no sustituyen una vuelta jugada.

En la ventana se verificaron: cuenta regresiva, paso de las dos primeras puertas
con W, frenado, pausa, ayuda H y cambios separados de volumen (música 28→21,
efectos 55→48). La pista local cargó y se indicó F1. Se observaron 56–58 FPS
en esas vistas. No se afirma una vuelta manual completa ni escucha subjetiva del audio.

La escala espacial se amplió diez veces manteniendo el tamaño de la nave y sus
velocidades. Las instalaciones se apoyan en la intersección radial con los triángulos
del LOD cercano para evitar separaciones entre base y terreno.

### Optimización de campos estáticos

Con las mismas escenas y 97 cuadros medidos después de calentamiento, precalcular
los dos campos de ruido de Mónaco redujo la media CPU de superficie de 16.62 a
11.34 ms y la del puerto de 15.86 a 10.99 ms. La diferencia media por canal de color
en la captura de superficie fue 0.064 sobre 255. El terreno, iluminación, pista,
mapa de normales y animación siguen ejecutándose en el pipeline propio de CPU.

En inicio, LOD + culling redujeron los triángulos enviados de 144,162 a 44,074;
en el circuito, de 147,618 a 35,990. Las salidas completas están en
`benchmark-etapa2.txt` y `benchmark-etapa2-campos.txt`.
