# Guía de defensa — Proyecto 3: Space Travel

Experiencia: **COSMOS / GP — ASTER R**.

Esta guía prepara la explicación. Leerla no sustituye abrir el código, cambiar un
parámetro y observar el resultado. La asistencia de IA está declarada; explica
con precisión lo que entiendes y puedes demostrar.

## Presentación de un minuto

«COSMOS / GP es un sistema solar explorable con una nave inspirada en F1. Conservé
Mónaco, Rebufo y la Nave Aster de mis trabajos previos y construí la navegación,
la perspectiva, las superficies y el contrarreloj alrededor de ellos, con asistencia
de Codex durante la implementación y las pruebas.

El mundo 3D se calcula en CPU. Yo controlo cómo se transforman, recortan y rasterizan
los triángulos, cómo se prueban las profundidades y cómo se calcula el color de
cada fragmento. Raylib abre la ventana, lee entradas, reproduce audio y muestra
el framebuffer ya calculado.

Puedo acercarme, rodear un planeta y salir de él sin cambiar de escena. Para mantener
la fluidez uso descarte de objetos, niveles de detalle y filas independientes de
rasterizado en varios hilos. Los contadores permiten comprobar que el trabajo cambia.»

Adapta estas palabras a tu comprensión real; no memorices afirmaciones que no puedas
sostener con una función y un ejemplo.

## Ruta del código

```mermaid
flowchart LR
  A[OBJ o geometría generada] --> B[Modelo a mundo]
  B --> C[Vista y proyección propias]
  C --> D[Clipping homogéneo]
  D --> E[Rasterización baricéntrica]
  E --> F[Profundidad e interpolación]
  F --> G[Shader CPU y mapa normal]
  G --> H[Framebuffer]
  H --> I[Textura 2D en la ventana]
```

| Parte | Archivo | Idea que debes explicar |
| --- | --- | --- |
| Carga y mallas | `src/mesh.rs` | OBJ triangulado, índices, materiales Kd; esferas, anillos y relieve propio. |
| Cámara | `src/camera.rs` | Vista inversa de la pose; matriz perspectiva escrita explícitamente. |
| Escena | `src/render.rs` | Modelo a mundo, selección de LOD, culling y envío de triángulos. |
| Pipeline | `src/framebuffer.rs` | Seis planos de clipping, coordenadas de pantalla, baricéntricas y depth test. |
| Materiales | `src/shader.rs` | Diferencias reales entre estrella, roca, gas, luna y anillos. |
| Detalle | `src/normal_map.rs` | Decodificar RGB a vector y pasarlo del espacio tangente al mundo. |
| Rendimiento | `src/material_field.rs` | Campos estáticos de ruido cacheados; luz y efectos dinámicos siguen por fragmento. |
| Vuelo | `src/simulation.rs` | Paso fijo, fuerzas, asistencia, gravedad, giro y colisiones. |
| Superficie | `src/surface.rs` | Modelos compuestos, distribución con semilla y orientación radial. |
| Carrera | `src/race.rs` | Cruce dirigido de planos, secuencia y persistencia del mejor tiempo. |
| Sonido | `src/audio.rs` | PCM sintetizado frente a una grabación local ajena al proyecto. |

## Ocho preguntas que debes poder contestar

**1. ¿Qué hace cada transformación?**
Modelo lleva un vértice del objeto al mundo: escala, rotación y traslación. Vista
expresa ese mundo desde la cámara. Proyección convierte un frustum en coordenadas
homogéneas. Después se divide por w y se aplica el viewport.

**2. ¿Por qué hay que recortar antes de dividir por w?**
Un triángulo que atraviesa la cámara puede tener vértices con w cercano a cero o
negativo. Proyectarlo directamente produce coordenadas enormes o inversiones.
Sutherland–Hodgman recorta contra ±x ≤ w, ±y ≤ w, ±z ≤ w y crea triángulos válidos.

**3. ¿Por qué la interpolación perspectiva usa 1/w?**
La proyección no conserva interpolación lineal de los atributos del mundo. Para
un atributo a y baricéntricas λ, se calcula:

`a = (Σ λᵢ · aᵢ / wᵢ) / (Σ λᵢ / wᵢ)`.

La profundidad NDC sí se interpola linealmente en pantalla. El shader solo se
evalúa si el fragmento está delante de lo que ya ocupa ese píxel; si descarta el
fragmento, no se actualiza la profundidad.

**4. ¿En qué se diferencian relieve y mapa de normales?**
El relieve mueve vértices y cambia la silueta. El mapa de normales cambia la normal
usada para iluminar un fragmento, conservando su posición. El mapa se genera como
RGB, se muestrea bilinealmente, se decodifica a [-1,1] y se transforma con T, B y N.
F6 muestra la diferencia; observar la silueta de Mónaco muestra el relieve real.

**5. ¿Cómo siguen los edificios al planeta?**
Cada instancia guarda una dirección local y un giro. Su base se apoya en la
intersección radial con la malla cercana. Su eje Y se alinea con la dirección radial
y esa pose se compone con la rotación del planeta. No se escriben posiciones
mundiales estáticas para cada edificio.

**6. ¿Cómo evita la simulación depender de los FPS?**
El tiempo transcurrido se acumula y se avanza en pasos de 1/120 s. Las entradas
representan fuerzas y velocidades angulares; no se suma una distancia fija por
cuadro. Las pruebas comparan el mismo vuelo a 30 y 144 FPS. La cámara interpola
con un factor exponencial dependiente del tiempo.

**7. ¿Qué hace el modo orbital?**
Sigue el desplazamiento del centro del planeta, alinea progresivamente el eje
vertical de la nave con la radial y separa movimiento tangencial y radial. Una
aceleración centrípeta mantiene la curva; R/F controlan la altura. Es una ayuda de
pilotaje diseñada para el juego, distinta de la gravedad libre de G.

**8. ¿Qué optimizaciones reducen trabajo real?**
Culling evita enviar objetos fuera del frustum u ocultos detrás del planeta.
LOD usa menos vértices y triángulos cuando el objeto ocupa menos pantalla.
El rasterizado paralelo reparte filas sin escrituras compartidas. El ruido de
material estático se precalcula con el mismo algoritmo y se muestrea después.
Ni la ventana ni los shaders del mundo usan dibujo 3D de Raylib.

## Cambiar parámetros en vivo

Primero cerrar la aplicación o la ejecución de Cargo, modificar el valor y volver
a ejecutar `cargo run --release --locked`. Mostrar el antes y el después.

| Cambio | Lugar | Resultado esperado |
| --- | --- | --- |
| Radio de Mónaco | Tupla de Mónaco en `world::planet`: `30.0` | Cambian tamaño, volumen de colisión, bases de edificios y escala relativa del acercamiento. Se multiplica por `SCALE`. |
| Velocidad orbital | La misma tupla: `0.006` | Cambia cuánto avanza el centro alrededor del sol por segundo. |
| Rotación propia | La misma tupla: `0.022` | Cambian orientación del terreno, instalaciones y puertas respecto del espacio. |
| Distribución | `world::SURFACE_SEED` | Otra distribución reproducible de los cuatro modelos; volver a la semilla restaura el patrón. |
| Intensidad del normal map | Multiplicadores `0.85` de `dx` y `dy` en `normal_map::initialize` | Aumenta o reduce el detalle de iluminación sin mover geometría. |
| Relieve geométrico | Amplitud `0.14` en `mesh::terrain_height` | Cambia la silueta. Si se aumenta mucho, también hay que ampliar el volumen conservador de colisión. |

Para explicar un cambio, mencionar qué se conserva y qué se modifica. Ejemplo:
«Al cambiar la semilla conservo los cuatro tipos de geometría y el algoritmo de
ruido; cambia qué posiciones superan el umbral de densidad y cómo se orientan».

## Ensayo de cinco minutos

1. Abrir la aplicación, mostrar H y realizar aproximación, O, R/F y alejamiento.
2. Mostrar la diferencia entre T asistido e inercial, y frenar con Espacio.
3. Pausar, activar F3 y alternar F4/F5 en la misma vista.
4. Mostrar los cuatro cuerpos y explicar cuatro shaders distintos.
5. Cambiar la semilla o velocidad orbital y explicar el resultado observable.
6. Señalar el clipping y la interpolación perspectiva en el código.

Si una parte todavía no la puedes explicar, empieza por ejecutar su prueba y seguir
la entrada hasta el resultado. El profesor puede pedir una modificación diferente
a las preparadas aquí.
