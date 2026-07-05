# MILESTONES.md — CLI en Tiempo Real por UART para ESP32

**Equipo:** Ekojoe Covenant Lemom (AlphaCode) y Juan Sebastian Valencia Londoño
**Repositorio:** <https://github.com/anidroid1184/rust_demo_day>
**Forma de trabajo:** Completamente asíncrona y remota. No se requieren sesiones en vivo. El progreso se mide por el estado de las ramas, no por el tiempo que pasamos conectados juntos.

---

## Cómo Trabajamos (Las Reglas Básicas)

- Cada hito tiene un **responsable** que lo lleva hasta el final.
- La otra persona es el **revisor** — lee el PR, deja comentarios, hace preguntas y aprueba antes de fusionar.
- Toda la comunicación ocurre por Telegram (asíncrono — sin esperar respuesta inmediata) o por comentarios en los PRs de GitHub.
- **Ninguno de los dos debería quedar bloqueado esperando que el otro esté en línea.** Si estás bloqueado, abre un PR en borrador y describe el problema en la descripción — el otro responderá cuando pueda.
- La rama `develop` siempre debe compilar. Nunca subas código roto a `develop`.
- `main` solo recibe fusiones al terminar un hito completo.

---

## Estrategia de Ramas

```plain
main         ← capturas estables, listas para demo (etiquetadas por hito)
develop      ← rama de integración, siempre compila
feat/uart-echo       ← Hito 1 (en progreso)
feat/interrupt-rx    ← Hito 2 (en progreso)
feat/ring-buffer     ← Hito 3 (no iniciado)
feat/command-parser  ← Hito 4 (no iniciado)
feat/polish          ← Hito 5 (no iniciado)
```

**Flujo de PRs:** `feat/*` → `develop` (revisado y aprobado por el otro) → al completar el hito, `develop` → `main` (con etiqueta).

---

## Hito 1 — Eco por UART (Bloqueante)

**Rama:** `feat/uart-echo`
**Responsable:** Ekojoe
**Revisor:** Juan
**Destino:** `develop`, luego etiquetar `main` como `v0.1.0`

**Objetivo:** Configurar UART0 en el ESP32. Leer un byte cuando llegue y devolverlo por la misma línea. Verificar en simulación de Wokwi usando el monitor serial.

**Por qué empezamos aquí:** La configuración del periférico (baud rate, pines TX/RX, ajustes del FIFO) es una habilidad separada del manejo de interrupciones. Comprobamos que el cableado de hardware funciona solo antes de añadir concurrencia encima.

**Definición de completado:**

- [ ] UART0 configurado con baud rate y pines correctos
- [ ] Eco bloqueante funcionando en simulación de Wokwi (escribes un carácter, lo ves de vuelta)
- [ ] Código revisado y aprobado por Juan
- [ ] Fusionado en `develop`
- [ ] `develop` fusionado en `main`, etiquetado `v0.1.0`

**Rol de Juan en este hito:** Revisar el PR. Leer el código de configuración de UART y asegurarte de entender cómo se calcula el baud rate y qué pines se usan — vas a construir encima de esto en el Hito 2.

---

## Hito 2 — Recepción por Interrupción

**Rama:** `feat/interrupt-rx`
**Responsable:** Juan
**Revisor:** Ekojoe
**Destino:** `develop`, luego etiquetar `main` como `v0.2.0`

**Objetivo:** Mover la recepción UART de bloqueante a manejada por interrupción. Cuando llega un byte, el periférico UART dispara una rutina de servicio de interrupción (ISR) que captura el byte y lo guarda en un buffer estático temporal. El bucle principal lee desde ese buffer.

**Por qué Juan es el responsable:** Juan tiene la placa ESP32 física. El comportamiento de las interrupciones en silicio real puede diferir de la simulación en formas sutiles — que el dueño de la placa maneje este hito significa que detectamos esas diferencias antes de fusionar.

**Por qué esto importa:** La recepción bloqueante significa que el bucle principal no puede hacer nada más mientras espera entrada. La recepción por interrupción libera el bucle principal para mantenerse activo — esto es lo esencial de que la CLI sea "en tiempo real."

**La parte difícil, dicho claramente:** Una ISR corre fuera del flujo normal del programa en un momento impredecible. Rust no permite una referencia mutable común a datos compartidos dentro de una ISR — y con buena razón, ya que crearía una condición de carrera. Para este hito usamos un `critical_section::Mutex<RefCell<Option<u8>>>` como estado compartido seguro temporal. El Hito 3 reemplaza esto con una solución sin bloqueos apropiada.

**Definición de completado:**

- [ ] Recepción UART manejada por ISR (no bloqueante)
- [ ] Bytes recibidos accesibles en el bucle principal mediante el buffer estático temporal
- [ ] Probado en la placa ESP32 física de Juan (no solo simulación)
- [ ] Código revisado y aprobado por Ekojoe
- [ ] Fusionado en `develop`
- [ ] `develop` fusionado en `main`, etiquetado `v0.2.0`

**Rol de Ekojoe en este hito:** Revisar el PR con atención. Enfocarse en entender el patrón de registro de la ISR y el uso de `critical_section` — este es el patrón que el Hito 3 reemplaza, y necesito entenderlo bien para escribir el reemplazo correctamente.

---

## Hito 3 — Buffer Circular Sin Bloqueos

**Rama:** `feat/ring-buffer`
**Responsable:** Ekojoe
**Revisor:** Juan
**Destino:** `develop`, luego etiquetar `main` como `v0.3.0`

**Objetivo:** Reemplazar el buffer estático temporal del Hito 2 con una división productor/consumidor de `bbqueue`. La ISR escribe bytes al handle `Producer`. El bucle principal lee bytes desde el handle `Consumer`.

**Por qué Ekojoe es el responsable:** Este hito es razonamiento puro de concurrencia en Rust — sin dependencia de hardware, completamente verificable en simulación de Wokwi.

**Por qué `bbqueue` específicamente:**

- La división productor/consumidor significa que la ISR y el bucle principal tienen cada uno su propio handle sin superposición al buffer. El sistema de tipos mismo evita el acceso concurrente — sin mutex, sin desactivar interrupciones, sin costo en tiempo de ejecución.
- `no_std` descarta estructuras basadas en heap como `VecDeque` a menos que agreguemos un allocator, lo cual añade complejidad innecesaria.
- Las implementaciones manuales de buffer circular frecuentemente tienen bugs sutiles en la lógica de "wraparound". `bbqueue` ya resolvió esto correctamente.

**Definición de completado:**

- [ ] `bbqueue` agregado a `Cargo.toml`
- [ ] Handle `Producer` pasado a la ISR, handle `Consumer` usado en el bucle principal
- [ ] Buffer estático temporal del Hito 2 completamente eliminado
- [ ] Simulación en Wokwi: los bytes escritos en el monitor serial llegan correctamente al bucle principal
- [ ] Código revisado y aprobado por Juan
- [ ] Fusionado en `develop`
- [ ] `develop` fusionado en `main`, etiquetado `v0.3.0`

**Rol de Juan en este hito:** Revisar el PR. Enfocarse en entender por qué la división productor/consumidor elimina la necesidad del mutex de `critical_section` del Hito 2 — este es un concepto clave para la defensa.

---

## Hito 4 — Parser de Comandos

**Rama:** `feat/command-parser`
**Responsable:** Ambos (dividido como se describe abajo)
**Destino:** `develop`, luego etiquetar `main` como `v0.4.0`

**Objetivo:** El bucle principal extrae bytes del `Consumer`, los acumula en un buffer de línea hasta ver un salto de línea (`\n`), separa por espacios, compara la primera palabra contra los comandos conocidos, y escribe una respuesta de vuelta por TX.

**División dentro de este hito:**

- **Ekojoe:** Acumulador de línea + andamiaje del despachador de comandos. Escribir las firmas de función y la tabla de despacho que mapea nombres de comandos a funciones manejadoras. Hacer commit de esto primero en la rama para que Juan tenga una interfaz clara contra la cual construir.
- **Juan:** Implementar los manejadores de comandos reales. Como mínimo: `ping` → responde `pong`, `help` → lista los comandos disponibles, `status` → responde con un mensaje corto de estado del chip.

**Por qué buffers de tamaño fijo:** `no_std` no tiene `String` basado en heap por defecto. Usamos `heapless::String<64>` o un arreglo plano `[u8; 64]` como buffer de línea — tamaño fijo, asignado en el stack, sin necesidad de allocator.

**Definición de completado:**

- [ ] Acumulador de línea funcionando (bytes se acumulan hasta `\n`)
- [ ] Al menos 3 comandos implementados y respondiendo correctamente (`ping`, `help`, `status`)
- [ ] Comando desconocido devuelve una respuesta de error clara
- [ ] Probado en simulación de Wokwi
- [ ] Ambos lados revisados y aprobados antes de fusionar
- [ ] Fusionado en `develop`
- [ ] `develop` fusionado en `main`, etiquetado `v0.4.0`

---

## Hito 5 — Pulido + Preparación para Demo

**Rama:** `feat/polish`
**Responsable:** Ambos
**Destino:** `develop` → `main`, etiquetado `v1.0.0-demo`

**Objetivo:** Dejar el proyecto listo para demo y defensa.

**Tareas:**

- [ ] Manejo de backspace en el acumulador de línea
- [ ] Eco de caracteres mientras el usuario escribe (la terminal se siente activa)
- [ ] Respuestas claras e informativas para todos los casos extremos
- [ ] `README.md` actualizado con instrucciones de configuración, instrucciones de demo y descripción del proyecto
- [ ] `ARCHITECTURE.md` completado con el diagrama de flujo de datos final y la justificación de las decisiones de diseño
- [ ] `diagram.json` de Wokwi limpio y presentable
- [ ] Video de demo grabado (1-2 minutos, mostrando la CLI respondiendo a comandos en simulación)
- [ ] Fusión final en `main`, etiquetado `v1.0.0-demo`

---

## Contrato de Interfaz (Acordado Antes de que Empiece el Hito 3)

Antes de que Juan empiece el Hito 2 y Ekojoe empiece el Hito 3, acordamos esta interfaz por escrito:

La ISR llamará:

```rust
// producer es un handle Producer de bbqueue accesible a la ISR
if let Ok(mut grant) = producer.grant_exact(1) {
    grant[0] = received_byte;
    grant.commit(1);
}
```

El bucle principal llamará:

```rust
if let Ok(grant) = consumer.read() {
    for byte in grant.buf() {
        // procesar byte
    }
    let len = grant.buf().len();
    grant.release(len);
}
```

Ambos hacemos commit de esta interfaz antes de que cualquier rama diverja significativamente — significa que Juan puede escribir el lado de la ISR y Ekojoe puede escribir el lado del consumidor sin necesidad de sincronizarse en tiempo real.

---

## Tabla Resumen

| Hito | Rama | Responsable | Revisor | Etiqueta en Main |
| ---- | ---- | ----------- | ------- | ---------------- |
| 1 — Eco UART | feat/uart-echo | Ekojoe | Juan | v0.1.0 |
| 2 — RX por Interrupción | feat/interrupt-rx | Juan | Ekojoe | v0.2.0 |
| 3 — Buffer Circular | feat/ring-buffer | Ekojoe | Juan | v0.3.0 |
| 4 — Parser | feat/command-parser | Ambos | Ambos | v0.4.0 |
| 5 — Pulido | feat/polish | Ambos | Ambos | v1.0.0-demo |
