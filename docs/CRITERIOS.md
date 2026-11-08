# Proyecto 3: Space Travel — criterios que se solicitan evaluar

La referencia es el PDF «Proyecto final de gráficos por computadora — Software Renderer».
Esta es una correspondencia entre implementación y evidencia, no una nota asignada.
La calificación y cualquier fracción de puntos corresponden al profesor.

## Cinco requisitos obligatorios

| Requisito | Evidencia |
| --- | --- |
| Pipeline propio de software | `camera.rs`, `render.rs`, `framebuffer.rs`, `shader.rs`. Raylib recibe píxeles terminados; no se invocan modelos, cámaras 3D, shaders GPU ni física de Raylib. |
| Sol, nave controlable y planeta que orbita y rota | Cinco Luces, Nave Aster OBJ y los dos planetas. `world.rs` usa tiempo de simulación; `simulation.rs` recibe entradas. |
| Acercamiento y alejamiento continuos | Exploración libre con W/S, O y R/F. El video completa acercamiento, vuelta y alejamiento sin reemplazar la escena. |
| Carga de modelos, MVP, profundidad y shader | `mesh::load_obj`, `Camera::matrix`, transformación de vértices en `Renderer::draw`, clipping/rasterización y buffer de profundidad. |
| Código, recursos, instrucciones y video | Carpeta de fuente, aplicación macOS con recursos, README y `COSMOS_GP_DEMO.mp4` en el paquete. |

## Menú de puntos

| Característica | Puntos solicitables | Dónde se observa / cómo se explica |
| --- | ---: | --- |
| Dos planetas diferenciados | 20 | Mónaco rojo/marfil y Rebufo gaseoso verde con anillos. Tab selecciona destino; ambos orbitan y rotan. La luna y el sol no se cuentan como planetas adicionales. |
| Cuatro tipos de modelo de superficie | 20 | Hangar, torre, antena y baliza. `surface::model` compone bloques, cilindros y un reflector. Vista `puerto`. |
| Distribución reproducible con ruido | 10 | `surface::distribute`, ruido de valor 3D y `world::SURFACE_SEED`. Misma semilla produce las mismas instancias. |
| Posición y orientación sobre superficie curva | 5 | Base calculada sobre la malla cercana; marco radial compuesto con la rotación del planeta. Prueba de orientación y seguimiento. |
| Cuatro shaders distintos | 24 | Plasma de la estrella, geología/circuito de Mónaco, bandas/huracán de Rebufo y cráteres de la luna en `shader.rs`. |
| Shader animado | 8 | Plasma y turbulencia de Rebufo dependen de `time`; se pueden observar sin mover la nave. |
| Mapeo de normales por software | 8 | Mapa RGB tangente, muestreo bilineal y transformación TBN en `normal_map.rs`. F6 permite comparar. |
| Deformación geométrica | 10 | `terrain_height` desplaza vértices de Mónaco antes de proyectar; cambia la silueta. |
| Generación procedural de geometría | 10 | Esferas de distinta teselación, anillos, aros y modelos compuestos se generan en CPU antes de rasterizar. |
| Modo de control local | 8 | O activa un marco radial y seguimiento del centro del planeta, sin cambiar de escena. |
| Controles progresivos y consistentes | 8 | Simulación fija a 120 Hz, aceleración y amortiguación temporal, giro independiente de velocidad. Prueba equivalente a 30 y 144 FPS. |
| Gravedad | 10 | En vuelo libre, G alterna la atracción radial inversamente proporcional al cuadrado de la distancia. Prueba compara trayectoria con y sin gravedad. |
| Colisión básica | 4 | Volúmenes esféricos impiden atravesar cuerpos celestes; respuesta a la velocidad entrante. Prueba de impulso hacia un planeta. |
| Descarte antes de rasterizar | 7 | Esferas frente al frustum y descarte de instalaciones ocultas por el planeta. F3 muestra contadores; F4 alterna culling. |
| Niveles de detalle | 8 | Cuatro teselaciones planetarias y modelos lejanos simplificados. F5 alterna LOD; cambian los triángulos enviados. |
| Coherencia visual | Hasta 5 | Paleta espacial, identidades de los cuerpos, Nave Aster R, instrumentos y señales del circuito. |
| Claridad de experiencia | Hasta 5 | H, ayudas contextuales, asistencia orbital, puertas secuenciales y controles de audio. |

**Máximo correspondiente a estas características: 170 puntos; la nota se limita a 100.**
Solo suma lo que funcione en la demostración y se pueda explicar. El contrarreloj,
el tema musical y el video cuidado apoyan la experiencia; no se les atribuyen puntos
adicionales que el menú no ofrece.

## Demostración breve de optimizaciones

Pausar con P, mostrar F3 y comparar F4/F5. Mantener la misma vista para comparar
contadores. En el caso medido de inicio, LOD + culling redujeron los triángulos
procesados de 144,162 a 44,074. El contador de píxeles no es igual a la resolución:
un píxel puede sombrearse más de una vez si un objeto posterior queda delante.
