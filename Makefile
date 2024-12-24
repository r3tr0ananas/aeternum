build:
	cargo build --release

install: install-shortcut
ifeq ($(detected_os), Windows)
	copy ".\target\release\aeternum.exe" "$(USERPROFILE)\.cargo\bin\"
else
	sudo cp ./target/release/aeternum /usr/bin/
endif

pull-submodules:
	git submodule update --init --recursive

update-submodules:
	git submodule update --recursive --remote