{
  inputs,
  lib,
  rustPlatform,

  cargo,
  rofi-unwrapped,
  pkg-config,
  glib,
  cairo,
  pango,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
  pname = cargoToml.package.name;
  version = cargoToml.package.version;
in rustPlatform.buildRustPackage rec {
  inherit pname version;

  src = builtins.path {
    path = lib.sources.cleanSource inputs.self;
    name = "${pname}-${version}";
  };

  strictDeps = true;

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [
    cargo
    pkg-config
  ];

  buildInputs = [
    glib
    cairo
    pango
    rofi-unwrapped
  ];

  postInstall = ''
    mkdir -p $out/lib/rofi
    mv $out/lib/*.so $out/lib/rofi
  '';

  meta = {
    description = cargoToml.package.description;
    homepage = "https://github.com/cyl3x/rofi-todo";
    license = lib.licenses.mit;
    maintainers = [];
  };
}
