let                                
   pkgs = import (builtins.fetchTarball {                                
      name = "nixos-20.09";                                
      url = "https://github.com/NixOS/nixpkgs/archive/20.09.tar.gz";                                
      sha256 = "1wg61h4gndm3vcprdcg7rc4s1v3jkm5xd7lw8r2f67w502y94gcy";                                
    }) {};                                
    src = builtins.fetchGit {                                
      url = "https://github.com/brest-computer-club/nix";                                
      ref = "master";                                
    };                                
    altPkgs = import src pkgs;                                
  in                                
  pkgs.mkShell {                                
    buildInputs = with pkgs;                                
       altPkgs.bundles.elm pkgs ++                                
       [ 
         pkgs.elmPackages.elm-test
         pkgs.nodePackages.uglify-js
         pkgs.rustup
     ];                                
  }      


