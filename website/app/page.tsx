import { DustMascot } from "./dust-mascot";
import { InstallCommand } from "./install-command";

const github = "https://github.com/y3l1n4ung/dust";
const description =
  "Dust is Rust-powered code generation for Dart and Flutter, built for human developers and AI coding agents. It handles repetitive and complex code so you can focus on your product.";

const features = [
  {
    name: "Data classes",
    status: "Stable",
    description: "Generate equality, copyWith, toString, and other typed model behavior.",
    href: github + "/blob/main/docs/usage/derive.md",
  },
  {
    name: "JSON",
    status: "Stable",
    description: "Typed serialization and deserialization without runtime reflection.",
    href: github + "/blob/main/docs/usage/serde.md",
  },
  {
    name: "Validation",
    status: "Stable",
    description: "Declare constraints beside your models and generate precise diagnostics.",
    href: github + "/blob/main/docs/usage/validation.md",
  },
  {
    name: "HTTP clients",
    status: "Stable",
    description: "Turn annotated interfaces into typed requests and generated tests.",
    href: github + "/blob/main/docs/usage/http.md",
  },
  {
    name: "Routing",
    status: "Beta",
    description: "Navigator 2.0 routes, deep links, guards, and typed results.",
    href: github + "/blob/main/docs/usage/routing.md",
  },
  {
    name: "State",
    status: "Beta",
    description: "Typed view models, scoped ownership, watch, and select APIs.",
    href: github + "/blob/main/docs/usage/state.md",
  },
  {
    name: "i18n",
    status: "Beta",
    description: "Scan translation keys, build ARB files, and generate typed access.",
    href: github + "/blob/main/docs/usage/i18n.md",
  },
  {
    name: "Database",
    status: "Beta",
    description: "sqlx-style checked SQL and generated row mapping for Dart.",
    href: github + "/blob/main/docs/usage/db.md",
  },
];

export default function Home() {
  return (
    <div className="site-frame">
      <header className="site-header">
        <nav className="shell nav" aria-label="Primary navigation">
          <a className="brand" href="#top" aria-label="Dust home">
            <DustMascot className="brand-mark" />
            <span>dust</span>
            <span className="brand-version">open source</span>
          </a>
          <div className="nav-links">
            <a href="#features">Features</a>
            <a href="#workflow">Workflow</a>
            <a href={github + "/tree/main/docs/usage"}>Docs</a>
            <a className="nav-status" href={github}>
              <span className="status-dot" aria-hidden="true" />
              GitHub
            </a>
          </div>
        </nav>
      </header>

      <main id="top">
        <section className="shell hero" aria-labelledby="hero-title">
          <div className="hero-content">
            <p className="eyebrow">Rust-powered code generation</p>
            <h1 id="hero-title">
              Built to make developers and <span>AI agents happy.</span>
            </h1>
            <p className="hero-copy">{description}</p>
            <div className="actions">
              <a className="button button-primary" href="#install">
                Install Dust <span aria-hidden="true">↓</span>
              </a>
              <a className="button" href={github}>
                View on GitHub <span aria-hidden="true">↗</span>
              </a>
            </div>
            <dl className="hero-facts" aria-label="Dust feature maturity">
              <div><dt>04</dt><dd>stable APIs</dd></div>
              <div><dt>04</dt><dd>beta surfaces</dd></div>
              <div><dt>MIT</dt><dd>licensed</dd></div>
            </dl>
          </div>

          <div className="terminal" aria-label="Real Dust 5000-file benchmark output">
            <div className="terminal-bar">
              <span className="terminal-title">~/dust/examples/benchmark_project</span>
              <span className="terminal-controls" aria-hidden="true">● ● ●</span>
            </div>
            <div className="terminal-body">
              <p><span className="prompt">$</span> dart run tool/generate.dart --count 5000</p>
              <p className="terminal-muted">generated 5000 source files in lib/generated_models</p>
              <p><span className="prompt">$</span> dust build --root examples/benchmark_project</p>
              <pre className="benchmark-output" aria-label="Dust cold and warm build results">{`    ____             __
   / __ \\__  _______/ /_
  / / / / / / / ___/ __/
 / /_/ / /_/ (__  ) /_
/_____/\\__,_/____/\\__/

build  scanned: 5010  generated: 5010  cached: 0  skipped: 0  time: 991ms

$ dust build --root examples/benchmark_project
build  scanned: 5010  generated: 3  cached: 5007  skipped: 0  time: 251ms`}</pre>
              <p className="terminal-cursor"><span className="prompt">$</span> <span /></p>
            </div>
            <div className="terminal-footer">
              <span><i aria-hidden="true" /> measured release build</span>
              <span>5,000 generated source files</span>
            </div>
          </div>
        </section>

        <section className="install-band" id="install" aria-labelledby="install-title">
          <div className="shell install-layout">
            <div>
              <p className="section-kicker">01 / Install</p>
              <h2 id="install-title">One command. Then build.</h2>
            </div>
            <InstallCommand />
          </div>
        </section>

        <section className="shell section" id="features" aria-labelledby="features-title">
          <div className="section-heading">
            <div>
              <p className="section-kicker">02 / Capabilities</p>
              <h2 id="features-title">A focused toolchain.</h2>
            </div>
            <p>Stable authoring APIs for core Dart work, with Flutter and database surfaces hardening in public.</p>
          </div>
          <div className="feature-grid">
            {features.map((feature, index) => (
              <a className="feature" href={feature.href} key={feature.name}>
                <div className="feature-top">
                  <span className="feature-index">{String(index + 1).padStart(2, "0")}</span>
                  <span className={"status " + feature.status.toLowerCase()}>{feature.status}</span>
                </div>
                <h3>{feature.name}</h3>
                <p>{feature.description}</p>
                <span className="feature-link">Read the guide ↗</span>
              </a>
            ))}
          </div>
        </section>

        <section className="workflow-section" id="workflow" aria-labelledby="workflow-title">
          <div className="shell section">
            <div className="section-heading">
              <div>
                <p className="section-kicker">03 / Workflow</p>
                <h2 id="workflow-title">Annotate. Generate. Ship.</h2>
              </div>
              <p>Normal Dart source stays readable. Generated implementation stays deterministic and reviewable.</p>
            </div>
            <ol className="workflow">
              <li>
                <span className="step-number">01</span>
                <div><h3>Annotate</h3><p>Describe the behavior beside the type you already own.</p></div>
                <pre><code><span className="code-accent">@Derive</span>([ToString(), CopyWith()]){"\n"}class User with _$User {"{"} … {"}"}</code></pre>
              </li>
              <li>
                <span className="step-number">02</span>
                <div><h3>Generate</h3><p>Run one fast command across the project or workspace.</p></div>
                <pre><code><span className="code-prompt">$</span> dust build{"\n"}<span className="code-ok">✓</span> user.g.dart</code></pre>
              </li>
              <li>
                <span className="step-number">03</span>
                <div><h3>Ship</h3><p>Use typed output and verify it stays current in CI.</p></div>
                <pre><code><span className="code-prompt">$</span> dust check{"\n"}<span className="code-ok">✓</span> current</code></pre>
              </li>
            </ol>
          </div>
        </section>

        <section className="shell packages" aria-labelledby="packages-title">
          <div>
            <p className="section-kicker">04 / Runtime packages</p>
            <h2 id="packages-title">Use only what your app needs.</h2>
          </div>
          <a className="package" href={github + "/tree/main/packages/dust_dart"}>
            <code>dust_dart</code>
            <span>Data classes · JSON · validation · HTTP · DB annotations</span>
            <b aria-hidden="true">↗</b>
          </a>
          <a className="package" href={github + "/tree/main/packages/dust_flutter"}>
            <code>dust_flutter</code>
            <span>Routing · state management · internationalization</span>
            <b aria-hidden="true">↗</b>
          </a>
        </section>

        <section className="shell open-source">
          <DustMascot
            className="open-source-mark"
            label="Happy Ferris holding the Flutter and Dart logos"
          />
          <div>
            <p className="section-kicker">Built in the open</p>
            <h2>Inspect every generated line.</h2>
            <p>Dust is MIT licensed. Explore the engine, run the examples, open an issue, or help shape the next stable API.</p>
          </div>
          <div className="open-source-actions">
            <a className="button button-primary" href={github + "/blob/main/CONTRIBUTING.md"}>Contribute ↗</a>
            <a className="button" href={github + "/tree/main/examples"}>Explore examples ↗</a>
          </div>
        </section>
      </main>

      <footer className="site-footer">
        <div className="shell footer-layout">
          <a className="brand" href="#top"><DustMascot className="brand-mark" /><span>dust</span></a>
          <p>Built to make developers and AI agents happy.</p>
          <div>
            <a href={github + "/releases"}>Releases</a>
            <a href={github + "/issues"}>Issues</a>
            <a href={github + "/security"}>Security</a>
            <a href="https://rustacean.net/">Happy Ferris artwork</a>
          </div>
        </div>
        <p className="shell trademark-notice">
          Flutter and the related logo are trademarks of Google LLC. Dart and the related logo are trademarks of Google LLC. Dust is not endorsed by or affiliated with Google LLC.
        </p>
      </footer>
    </div>
  );
}
