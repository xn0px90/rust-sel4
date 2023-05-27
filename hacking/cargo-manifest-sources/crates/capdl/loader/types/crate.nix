{ mk, localCrates, serdeWith, postcardWith }:

mk {
  package.name = "capdl-loader-types";
  dependencies = {
    capdl-types.features = [ "alloc" "serde" "deflate" ];
    postcard = postcardWith [ "alloc" ];
    serde = serdeWith [ "alloc" ];
  };
  nix.local.dependencies = with localCrates; [
    capdl-types
  ];
}