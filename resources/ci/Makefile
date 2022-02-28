NAMESPACE = fujiapple
REPOSITORY = rust-daml
TAG = 1.59.0-1_18_1-v1
.PHONY: build push
build:
	docker build --network host -t ${NAMESPACE}/${REPOSITORY}:${TAG} -t ${NAMESPACE}/${REPOSITORY}:latest .
push:
	docker push ${NAMESPACE}/${REPOSITORY}:${TAG}
