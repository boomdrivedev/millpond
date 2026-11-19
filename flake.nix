# millpond as a nix package, for `nix profile install github:boomdrivedev/millpond`
# and for a host's configuration. The development shell is devenv's
# (devenv.nix). Rust is the version .prototools names, so the release is
# built by the compiler the checks ran with. The package's version is the
# tree's, Cargo.toml's [workspace.package]: 0.0.0 on main, and the
# release's at a release's tag. The binary says the revision it was built
# from, which this gives it, and the build that made it. recutils' recfix
# is on its PATH: weir's write protocol holds every record to it (C.2).
{
  description = "millpond: the inbox for events that should change a repository's work";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-linux"
      ];
      forAll =
        f:
        nixpkgs.lib.genAttrs systems (
          system:
          f (
            import nixpkgs {
              inherit system;
              overlays = [ rust-overlay.overlays.default ];
            }
          )
        );
      rustVersion = (builtins.fromTOML (builtins.readFile ./.prototools)).rust;
      workspace = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package;
      revision = self.rev or self.dirtyRev or "unknown";
    in
    {
      packages = forAll (
        pkgs:
        let
          rust = pkgs.rust-bin.stable.${rustVersion}.minimal;
          platform = pkgs.makeRustPlatform {
            cargo = rust;
            rustc = rust;
          };
        in
        rec {
          millpond = platform.buildRustPackage {
            pname = "millpond";
            inherit (workspace) version;
            env.MILLPOND_REVISION = revision;
            src = self;
            cargoLock = {
              lockFile = ./Cargo.lock;
              allowBuiltinFetchGit = true;
            };
            cargoBuildFlags = [
              "-p"
              "millpond"
            ];
            nativeBuildInputs = [ pkgs.makeWrapper ];
            postInstall = ''
              wrapProgram $out/bin/millpond --prefix PATH : ${
                pkgs.lib.makeBinPath [
                  pkgs.recutils
                  pkgs.git
                ]
              }
            '';
            # The tests run in the devenv shell, through moon.
            doCheck = false;
            meta = {
              description = "The inbox for events that should change a repository's work";
              homepage = "https://github.com/boomdrivedev/millpond";
              license = pkgs.lib.licenses.mit;
              mainProgram = "millpond";
            };
          };
          default = millpond;
        }
      );
    };
}
