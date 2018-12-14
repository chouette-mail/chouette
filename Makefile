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
	@mkdir -p $(BUILD_DIR)
	@cp $(STATIC_FILES) $(BUILD_DIR)

client-dev: client/src/**
	@cd client && $(ELM) make src/Main.elm --output ../$(BUILD_DIR)/main.js

client-watch:
	@cd client && $(ELMLIVE) src/Main.elm -d build -- --output ../$(BUILD_DIR)/main.js

server-dev:
	@cd server && cargo build

clean-client:
	rm -rf $(BUILD_DIR)
