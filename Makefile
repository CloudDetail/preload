.PHOHY: preload-lib
preload-lib: outputdir launcher-lib instrument-lib

.PHOHY: dist
dist: preload-lib
	@mkdir -p apo-instrument
	@cp -rf build/* apo-instrument/.
	@cp -rf apo.ld.so.preload apo-instrument/
	@cp -rf agent apo-instrument/.
	@make instrument-conf
	@echo "use install.sh to install"

.PHOHY: release
release: dist
	tar -czvf install-apo-instrument.tar.gz apo-instrument install.sh uninstall.sh
	@echo "use install-apo-instrument.tar.gz to install"

outputdir:
	mkdir -p build

launcher-lib:
	@echo "Generate libapolanucher.so"
	gcc -std=c99 -o build/libapolanucher.so -shared launcher/lanucher.c -Wall -Wfatal-errors -fPIC -ldl

instrument-lib:
	@echo "Generate libapoinstrument.so"
	cd instrument && cargo build --release  && cp -f target/release/libapoinstrument.so ../build/.

.PHOHY: instrument-conf
instrument-conf:
	@echo "Generate libapoinstrument.conf"
	@cp instrument_conf_template apo-instrument/libapoinstrument.conf
	@for dir in apo-instrument/agent/*/ ; do \
		( \
			cd $$dir ; \
			if [ -f check_download.sh ]; then \
				bash check_download.sh && cat agent.conf >> ../../libapoinstrument.conf; \
				echo "" >> ../../libapoinstrument.conf; \
			fi ; \
		) \
	done


