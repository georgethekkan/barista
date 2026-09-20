# Barista Project Roadmap & Prioritized Task List

This roadmap organizes the remaining work for Barista into prioritized tiers, starting with the blockers required to complete the core build-and-run workflow.

---

## 🚨 P0: Critical Path (Make the Core Loop Work)

These items must be completed before Barista can reliably compile and run non-trivial Java applications.

- [x] **Classpath Construction (`src/build.rs` & `src/run.rs`)**
  - Replace `jar_paths.push("fix_me")` in `src/build.rs`.
  - Collect all `.jar` files in `target/dependencies/` (including transitives) or compute paths from resolved dependencies.
  - Combine dependencies with `target/classes` in `src/run.rs` using `std::env::join_paths`.
- [x] **Dynamic Java Source Discovery (`src/build.rs`)**
  - Replace hardcoded `"src/main/java/Main.java"`.
  - Use `walkdir` to recursively locate all `*.java` files under `src/main/java/`.
- [ ] **Directory Pre-Creation**
  - Ensure `target/classes` and `target/dependencies` exist before invoking `javac` or Ristretto (`std::fs::create_dir_all`).

---

## ⚡ P1: Reliability & Correctness

Prevent runtime panics and ensure deterministic dependency handling.

- [x] **Safe Exclusion Parsing (`src/manifest.rs`)**
  - Replaced `.unwrap()` with `TryFrom<String> for Exclusion` returning `Result`.
- [x] **Direct Deserialization for Manifest (`src/manifest.rs`)**
  - Deserialized directly into `BTreeMap<String, Dependency>` via `#[serde(try_from = "...")]`, eliminating `src/deps.rs`.
- [x] **Slice Idioms (`&[T]` instead of `&Vec<T>`)**
  - Updated `src/resolver.rs` (`&Vec<Dependency>` -> `&[Dependency]`).
- [ ] **Pass Configured Java Version to `javac`**
  - If `manifest.package.java_version` is present, pass `--release <version>` to `javac`.

---

## 🛠️ P2: Developer Experience & Ergonomics

Polish the command-line experience and environment resilience.

- [ ] **JDK Detection & `JAVA_HOME` Support**
  - Check `env::var("JAVA_HOME")` to find `javac` and `java` binaries before falling back to system `PATH`.
- [ ] **Project Root / Manifest Discovery**
  - Currently hardcoded to `./barista.toml`. Support searching parent directories upward (like Cargo) so running `barista` from subdirectories works.
- [ ] **Compiler Error Output Formatting**
  - When `javac` fails, print clear compiler error lines directly without raw debug formatting.
- [x] **Clean Tracing Output in `resolver.rs`**
  - Remove debug `println!("{:?}", art.path)` in `src/resolver.rs` and replace with `tracing::debug!`.

---

## 🚀 P3: Future Milestones

Features that take Barista from a basic runner to a full build system.

- [ ] **`barista test` Subcommand**
  - Discover `src/test/java/**/*.java`.
  - Compile with test dependencies (e.g. JUnit Jupiter) and invoke a console test launcher.
- [ ] **Java `@argfile` Support**
  - For projects with hundreds of Java files or JARs, write paths to a temporary file (`@sources.txt`) to avoid OS command-line length limits.
- [ ] **`barista package` / JAR Packaging**
  - Add an archive step to package compiled classes into a runnable `.jar` with `META-INF/MANIFEST.MF`.
