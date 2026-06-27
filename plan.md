# Plan de Proyecto: CLI en Tiempo Real por UART para ESP32

## 1. Metadatos del Proyecto
* **Equipo:** Ekojoe Covenant Lemom & Juan Sebastian Valencia Londoño
* **Repositorio:** [github.com/anidroid1184/rust_demo_day](https://github.com/anidroid1184/rust_demo_day)
* **Target Arquitectura:** `xtensa-esp32-none-elf` (ESP32 Clásico, Xtensa LX6)
* **Framework:** `esp-hal` (Ambiente bare-metal, `no_std`)
* **Herramienta de Simulación:** Wokwi (Extensión de VS Code)

---

## 2. Objetivo del Proyecto
Diseñar e implementar una interfaz de línea de comandos (CLI) en tiempo real para el microcontrolador ESP32 corriendo en Rust bajo un entorno bare-metal (`no_std`). La entrada de datos por UART se capturará mediante una interrupción de hardware que alimentará un buffer circular síncrono e indexado libre de bloqueos (`bbqueue`), liberando al ciclo principal (`main loop`) para el procesamiento semántico de comandos y el despacho de respuestas a través de la misma línea serial.

---

## 3. Arquitectura del Repositorio (`Tree`)
El proyecto mantendrá una separación estricta de responsabilidades (Hardware, Concurrencia, Lógica) para facilitar el desarrollo en paralelo y asegurar la modularidad requerida para la defensa técnica:

```plain
rust_demo_day/
├── .cargo/
│   └── config.toml          # Configuración del linker y argumentos de compilación para Xtensa
├── docs/
│   └── ARCHITECTURE.md      # Documentación detallada del diseño y decisiones de memoria
├── src/
│   ├── command_parser.rs    # Parseo de cadenas de texto y despacho de comandos (heapless)
│   ├── main.rs              # Punto de entrada de la aplicación y ciclo principal
│   ├── ring_buffer.rs       # Abstracción y configuración de bbqueue (Lock-free memory)
│   └── uart_handler.rs      # Inicialización de UART0 y vector de interrupciones de hardware
├── .gitignore
├── Cargo.toml               # Manifiesto de dependencias (esp-hal, bbqueue, heapless)
├── build.rs                 # Script de compilación para enlazar los mapas de memoria del chip
├── diagram.json             # Configuración de conexiones de hardware en Wokwi
├── README.md                # Manual de usuario y bitácora del proyecto
├── rust-toolchain.toml      # Canal y versión específica del compilador de Rust (Xtensa fork)
└── wokwi.toml               # Configuración del entorno de simulación integrado
```

## 4. Configuración del Entorno de Desarrollo
Comandos iniciales obligatorios para preparar el toolchain de Rust para Xtensa y las herramientas de flasheo/simulación en la máquina local:
```bash
# 1. Instalar el gestor del toolchain para arquitecturas de Espressif
cargo install espup
espup install

# 2. Instalar herramientas de generación y flasheo nativo
cargo install esp-generate
cargo install espflash

# 3. Inicializar el entorno (Estructura de compilación y scripts del linker correctos)
esp-generate --chip esp32 -o vscode rust_demo_day
```

Nota: Para habilitar la simulación local en VS Code, abrir la paleta de comandos con F1 y ejecutar Wokwi: Request a new License.



## 5. Cronograma de Desarrollo y Justificación Técnica (`no_std`)

### Paso A: Eco por UART (Bloqueante, Sin Interrupciones)
* **Por qué:** Validar que el periférico UART0 (pines de transmisión/recepción TX/RX, baud rate) esté correctamente inicializado en el hardware o simulador antes de introducir asincronía. Depurar fallas de hardware mezcladas con fallas en vectores de interrupción eleva exponencialmente la dificultad del diagnóstico.
* **Módulo asociado:** `uart_handler.rs` (Configuración del periférico).

### Paso B: Recepción Guiada por Interrupciones
* **Por qué:** Evitar la pérdida de bytes y la degradación del rendimiento. El periférico genera una interrupción de hardware inmediatamente al recibir un byte en la FIFO de lectura, despertando al procesador para extraer el dato de inmediato sin bloquear el hilo de ejecución principal.
* **Módulo asociado:** `uart_handler.rs` (Vector de interrupción mediante `esp-hal`).

### Paso C: Buffer Circular Sin Bloqueos (`bbqueue`)
* **Por qué:** Transferir datos de forma segura desde el contexto de la interrupción (Productor) al contexto del ciclo principal (Consumidor) sin usar *Mutexes* globales que requieran deshabilitar interrupciones críticamente. `bbqueue` implementa buffers basados en arreglos estáticos contiguos adecuados para `no_std`, garantizando la seguridad de memoria de Rust mediante separación de accesos atómicos.
* **Módulo asociado:** `ring_buffer.rs`.

### Paso D: Analizador Léxico y Parser de Comandos
* **Por qué:** En entornos embebidos puros no se dispone de un asignador dinámico de memoria (*heap*) por defecto. Se procesa la entrada acumulando bytes en buffers estáticos de tamaño fijo provistos por la crate `heapless` hasta detectar un salto de línea (`\n` o `\r`), dividiendo luego los argumentos para su ejecución.
* **Módulo asociado:** `command_parser.rs`.

### Paso E: Pulido de Interfaz y UX de la CLI
* **Por qué:** Ofrecer una experiencia interactiva correcta manejando caracteres de control comunes de terminales (procesamiento de *Backspace* `0x08` o `0x7F`), retroalimentación de caracteres por eco local inmediato y respuestas estructuradas ante comandos inexistentes.

---

## 6. Matriz de Asignación de Roles (RACI)

Para maximizar el aprendizaje y mitigar cuellos de botella en los fundamentos, los hitos iniciales se realizarán bajo programación en pareja (*Pair Programming*), procediendo luego con la división modular especializada.

| Fase / Módulo | Juan S. Valencia | Ekojoe C. Lemom | Tipo de Trabajo |
| :--- | :---: | :---: | :--- |
| **Paso A & B** (UART & Interrupciones) | **R** / **A** | **R** / **A** | Colaborativo (Pair Programming) |
| **Paso C** (`ring_buffer.rs` - bbqueue) | **R** | **C** | Individual (Especializado) |
| **Paso D** (`command_parser.rs` - CLI) | **C** | **R** | Individual (Especializado) |
| **Paso E & Integración Final** | **R** / **A** | **R** / **A** | Colaborativo (Pruebas de Sistema) |

* **R (Responsible):** Ejecutor directo del código asignado.
* **A (Accountable):** Responsable final de asegurar la calidad y correcto funcionamiento del entregable.
* **C (Consulted):** Compañero de equipo que asiste aportando ideas, revisión lógica y pruebas cruzadas.

---

## 7. Flujo de Trabajo en Git y Gestión de Ramas
* **Rama Principal (`main`):** Se mantiene estrictamente estable y compilable. Queda prohibido realizar Commits directos sobre esta rama.
* **Estrategia de Ramificación:** Se creará una rama por cada funcionalidad técnica partiendo siempre de `main`:
  * `feature/uart-echo`
  * `feature/interrupt-rx`
  * `feature/bbqueue-integration`
  * `feature/command-parser`
* **Políticas de Integración:** Antes de realizar un *Merge* hacia `main`, el código debe ser expuesto al otro integrante del equipo. Explicar el código mutuamente servirá como preparación activa para la defensa del proyecto.

---

## 8. Hitos y Criterios de Aceptación (Definition of Done)

1. **Hito 1: Parpadeo Inicial e Infraestructura**
   * *Criterio de Aceptación:* Toolchain configurado correctamente, compilación sin advertencias y un LED parpadeando en la simulación de Wokwi a través de `esp-hal`.
2. **Hito 2: Canal de Comunicación Estable**
   * *Criterio de Aceptación:* El ESP32 devuelve exactamente el mismo caracter enviado por la consola serial en modo bloqueante.
3. **Hito 3: Asincronía por Hardware**
   * *Criterio de Aceptación:* Los bytes entrantes disparan la rutina de servicio de interrupción (ISR) de la UART sin colgar el ciclo principal del microcontrolador.
4. **Hito 4: Integración del Buffer de Memoria Seguro**
   * *Criterio de Aceptación:* La ISR deposita exitosamente los bytes en el `Producer` de la `bbqueue` y el ciclo principal los extrae desde el `Consumer` sin pérdida de datos ni condiciones de carrera (*Race Conditions*).
5. **Hito 5: Demostración Mínima Viable (MVD)**
   * *Criterio de Aceptación:* El parser interpreta y procesa de forma correcta al menos 3 comandos reales con sus argumentos (ej. `help`, `status`, `led <on/off>`), retornando los resultados por la terminal serial.
6. **Hito 6: Cierre de Entrega**
   * *Criterio de Aceptación:* Código final refactorizado, documentación del repositorio actualizada en `README.md` y `docs/ARCHITECTURE.md`, y video demostrativo de funcionamiento grabado.
