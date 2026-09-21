# Barista Project Roadmap & Prioritized Task List

This roadmap organizes the remaining work for Barista into prioritized tiers.

---

## 🚨 P0: Core Robustness & Blockers

Items required to guarantee dependable builds across environments.

- [ ] **Directory Pre-Creation**
  - Ensure `target/classes` and `target/dependencies` exist before invoking `javac` or Ristretto (`std::fs::create_dir_all`).
- [ ] **Forward CLI Arguments in `barista run`**
  - Pass `bin_args` (arguments passed after `--`) down to the executed Java application.

---

## ⚡ P1: Reliability & Correctness

Prevent build inconsistencies and respect project configuration.

- [ ] **Pass Configured Java Version to `javac`**
  - If `manifest.package.java_version` is present, pass `--release <version>` to `javac`.
- [ ] **Project Root / Manifest Discovery**
  - Instead of hardcoding `./barista.toml`, support searching parent directories upward (like Cargo) so running `barista` from subdirectories works.

---

## 🛠️ P2: Developer Experience & Ergonomics

Polish the command-line experience and environment resilience.

- [ ] **JDK Detection & `JAVA_HOME` Support**
  - Check `env::var("JAVA_HOME")` to find `javac` and `java` binaries before falling back to system `PATH`.
- [ ] **Compiler Error Output Formatting**
  - When `javac` fails, print clean compiler diagnostics without log prefix noise.
- [ ] **Filter Jars in Classpath**
  - Filter `target/dependencies` strictly for `.jar` files when building the classpath.

---

## 🚀 P3: Future Milestones

Features that take Barista from a basic runner to a full build tool.

- [ ] **`barista test` Subcommand**
  - Discover `src/test/java/**/*.java`.
  - Compile with test dependencies (e.g. JUnit Jupiter) and invoke a console test launcher.
- [ ] **Java `@argfile` Support**
  - For projects with hundreds of Java files or JARs, write paths to a temporary file (`@sources.txt`) to avoid OS command-line length limits.
- [ ] **`barista package` / JAR Packaging**
  - Add an archive step to package compiled classes into a runnable `.jar` with `META-INF/MANIFEST.MF`.
