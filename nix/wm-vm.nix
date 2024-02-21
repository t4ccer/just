{justwindows}: {
  config,
  modulesPath,
  pkgs,
  lib,
  ...
}: {
  imports = ["${modulesPath}/virtualisation/qemu-vm.nix"];
  virtualisation = {
    memorySize = 8192;
    diskSize = 100000;
  };

  networking.firewall.enable = false;

  services.getty.autologinUser = "root";

  services.openssh = {
    enable = true;
    settings.PermitRootLogin = "yes";
  };

  users = {
    extraUsers.root.password = "root";
    mutableUsers = false;
  };

  services.xserver = {
    enable = true;
    xkb.layout = "us";

    displayManager = {
      lightdm.enable = true;
    };

    windowManager.session = lib.singleton {
      name = "justwindows";
      start = ''
        ${justwindows}/bin/justwindows &
        waitPID=$!
      '';
    };
  };

  environment.systemPackages = [
    pkgs.rofi
  ];

  system.stateVersion = "24.05";
}
