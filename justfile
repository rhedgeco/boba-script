default:
    nix develop -c ${SHELL}

code:
    nix develop -c ${SHELL} -c "code ."