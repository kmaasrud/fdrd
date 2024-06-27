SRC_DIR = src
PUBLIC_DIR = public
PUBLIC := $(PUBLIC_DIR)/index.html

.PHONY: all
all: $(PUBLIC)

$(PUBLIC_DIR)/index.html: $(SRC_DIR)/index.html
	mkdir $(PUBLIC_DIR) || true
	cp $< $@

.PHONY: clean
clean:
	rm -rf $(PUBIC_DIR)
