self: with self; {

  worldsForEverythingInstances = [
    pkgs.host.aarch64.none.this.worlds.default
    pkgs.host.x86_64.none.this.worlds.default
  ];

  everythingList = lib.flatten [
    (lib.forEach worldsForEverythingInstances (world:
      map (instance: instance.links) world.instances.supported
    ))
    pkgs.host.riscv64.none.this.worlds.default.kernel
  ];

  everythingWithExcessList = lib.flatten [
    everything
    html
  ];

  everything = pkgs.build.writeText "everything" (toString everythingList);
  everythingWithExcess = pkgs.build.writeText "everything" (toString everythingWithExcessList);

  runAutomatedTests = mkRunAutomatedTests
    (lib.flatten
      (lib.forEach worldsForEverythingInstances (world:
        lib.forEach (lib.filter (instance: instance.canAutomate) world.instances.supported) (instance: {
          name = "unnamed"; # TODO
          script = instance.automate;
        })
      ))
    );

  mkRunAutomatedTests = tests:
    with pkgs.build;
    writeScript "run-all" ''
      #!${runtimeShell}
      set -e

      ${lib.concatStrings (lib.forEach tests ({ name, script }: ''
        echo "<<< running case: ${name} >>>"
        ${script}
      ''))}

      echo
      echo '# All tests passed.'
      echo
    '';

  docs = import ./docs.nix {
    inherit lib pkgs;
  };

  inherit (docs) html;

  # convenience

  worlds = lib.fix (self: {
    default = self.aarch64.default;
  } // lib.listToAttrs
    (lib.forEach
      [ "aarch64" "riscv64" "x86_64" ]
      (arch: lib.nameValuePair arch (pkgs.host.${arch}.none.this.worlds)))
  );

  example = worlds.default.instances.examples.full-runtime.simulate;

}