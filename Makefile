SRC_DIR = src
PUBLIC_DIR = public
PUBLIC := $(PUBLIC_DIR)/index.html $(PUBLIC_DIR)/main.css $(PUBLIC_DIR)/main.js

.PHONY: all
all: $(PUBLIC)

.PHONY: serve
serve: $(PUBLIC)
	python3 -m http.server -d $(PUBLIC_DIR)

$(PUBLIC_DIR)/%.html: $(SRC_DIR)/%.html
	minify $< -o $@

$(PUBLIC_DIR)/%.css: $(SRC_DIR)/%.css
	lightningcss -m $< -o $@

$(PUBLIC_DIR)/%.js: $(SRC_DIR)/%.js
	uglifyjs $< -o $@

.PHONY: clean
clean:
	rm -rf $(PUBLIC_DIR)
