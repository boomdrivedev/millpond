_: {
  # A tracked file no formatter claims and no exclude names fails the check:
  # a new file type is decided in the change that adds it.
  settings.on-unmatched = "fatal";

  # What no formatter owns, each with its reason.
  settings.excludes = [
    # Prose: the README and the site's pages are Vale's (`prose`), and no
    # formatter here rewraps prose.
    "*.md"
    "*.mdx"
    # Written by devenv and bun in their own layouts.
    "*.lock"
    # The license text, kept as issued.
    "LICENSE"
    # Lists in gitignore syntax, Vale's own configuration and its word
    # lists, which no formatter reads.
    ".gitignore"
    ".worktreeinclude"
    ".vale.ini"
    "styles/config/vocabularies/**"
    # The site is its own project, held to its build (site/moon.yml), and
    # the README's diagram is written from it (`bun run readme-diagram`).
    "site/**"
    "docs/diagrams/**"
    # Build outputs and caches.
    "target/**"
    ".devenv/**"
    ".moon/cache/**"
  ];

  # The crate's Rust, with the pinned toolchain's rustfmt (devenv.nix).
  programs.rustfmt.enable = true;
  programs.rustfmt.edition = "2021";

  # Cargo.toml, the crate's and every other TOML file.
  programs.taplo.enable = true;

  programs.deadnix.enable = true;
  programs.nixfmt.enable = true;
  settings.formatter.deadnix.pipeline = "nix";
  settings.formatter.deadnix.priority = 1;
  settings.formatter.nixfmt.pipeline = "nix";
  settings.formatter.nixfmt.priority = 2;

  # .prototools is TOML and .versionrc convco's YAML, under names with no
  # extension.
  settings.formatter.taplo.includes = [
    "*.toml"
    ".prototools"
  ];
  programs.yamlfmt.enable = true;
  settings.formatter.yamlfmt.includes = [ ".versionrc" ];
  programs.yamlfmt.settings.formatter = {
    type = "basic";
    retain_line_breaks_single = true;
  };
}
