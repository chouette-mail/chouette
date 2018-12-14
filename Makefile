ifeq ("$(ELM)","")
	ELM=elm
endif

ifeq ("$(ELMLIVE)", "")
	ELMLIVE=elm-live
endif

STATIC_FILES=client/static/*
BUILD_DIR=./server/dist

all: client-dev client-static server-dev

client-static: client/static/*
	@echo -e "\033[32;1m     Copying\033[0m chouette-client static files"
	@mkdir -p $(BUILD_DIR)
	@cp $(STATIC_FILES) $(BUILD_DIR)
	@echo -e "\033[32;1m      Copied\033[0m chouette-client static files"

client-dev: client/src/**
	@echo -e "\033[32;1m   Compiling\033[0m chouette-client"
	@cd client && $(ELM) make src/Main.elm --output ../$(BUILD_DIR)/main.js
	@echo -e "\033[32;1m    Finished\033[0m chouette-client"

client-watch:
	@echo -e "\033[32;1m    Watching\033[0m chouette-client"
	@cd client && $(ELMLIVE) src/Main.elm -d build -- --output ../$(BUILD_DIR)/main.js

server-dev:
	@cd server && cargo build

clean-client:
	@echo -e "\033[32;1m    Cleaning\033[0m chouette-client"
	@rm -rf $(BUILD_DIR)
	@echo -e "\033[32;1m     Cleaned\033[0m chouette-client"

clean: clean-client
