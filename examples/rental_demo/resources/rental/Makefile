DAML_LF_VERSION = 1.8
OUTPUT_PATH = .daml/dist
DAR_NAME = rental

SDK_VERSION := $(shell daml version | grep project | cut -d" " -f 3)
DAR_VERSION := $(shell egrep "^version" daml.yaml | cut -d" " -f 2)
OUTPUT_DAR_NAME := $(shell echo ${DAR_NAME}-${DAR_VERSION}-sdk_${SDK_VERSION}-lf_${DAML_LF_VERSION} | sed "s/\./_/g")
OUTPUT_FULL = ${OUTPUT_PATH}/${OUTPUT_DAR_NAME}.dar

build:
	daml build --target ${DAML_LF_VERSION} --output ${OUTPUT_FULL}

inspect:
	daml damlc inspect-dar ${OUTPUT_FULL}

run: build
	daml sandbox --port 8082 --contract-id-seeding=strong ${OUTPUT_FULL}