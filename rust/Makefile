
# First argument is the target os (either "linux" or "windows")
# Second argument is the architecture name
# Third argument is the rust target architecture
# Fourth argument is the build mode (either "debug" or "release")
define build_target =
	$(eval @_OS = $1)
	$(eval @_ARCH = $2)
	$(eval @_RUST_ARCH = $3)
	$(eval @_MODE = $4)

	$(eval @_ARGS = $(if $(findstring $(@_MODE), release),--release,))

	mkdir -p ../bin;

	echo cargo build --target $(@_RUST_ARCH) $(@_ARGS);
	cargo build --target $(@_RUST_ARCH) $(@_ARGS);

	if [ "$(@_OS)" = "linux" ]; then                                                                                   \
		cp target/$(@_RUST_ARCH)/$(@_MODE)/libcereal_object.so ../bin/libcereal_object.$(@_OS).$(@_MODE).$(@_ARCH).so; \
	fi;

	if [ "$(@_OS)" = "windows" ]; then                                                                                 \
		cp target/$(@_RUST_ARCH)/$(@_MODE)/cereal_object.dll ../bin/libcereal_object.$(@_OS).$(@_MODE).$(@_ARCH).dll;  \
	fi;
endef


debug: FORCE
## Windows builds
# x86_64
	@$(call build_target,windows,x86_64,x86_64-pc-windows-gnu,debug)

# Linux builds
# x86_64
	@$(call build_target,linux,x86_64,x86_64-unknown-linux-gnu,debug)


release: FORCE
## Windows builds
# x86_64
	@$(call build_target,windows,x86_64,x86_64-pc-windows-gnu,release)

# Linux builds
# x86_64
	@$(call build_target,linux,x86_64,x86_64-unknown-linux-gnu,release)


all: debug release ;



FORCE: ;