APP_NAME=julia-fractal-renderer

WIN_TARGET=x86_64-pc-windows-gnu
RELEASE_W=target/$(WIN_TARGET)/release
RELEASE_L=target/release

release: release_windows release_linux

release_linux:
	cargo build --release
	cd $(RELEASE_L) && tar -caf $(APP_NAME)-linux.tar.xz $(APP_NAME)
	mv $(RELEASE_L)/$(APP_NAME)-linux.tar.xz .

release_windows:
	cargo build --release --target $(WIN_TARGET)
	cd $(RELEASE_W) && zip -9 $(APP_NAME)-windows.zip $(APP_NAME).exe
	mv $(RELEASE_W)/$(APP_NAME)-windows.zip .

