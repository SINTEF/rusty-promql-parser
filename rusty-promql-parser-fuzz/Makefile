.PHONY: build fuzz create-in clean

build:
	cargo afl build

fuzz: create-in build
	cargo afl fuzz -i in -o out target/debug/rusty-promql-parser-fuzz

create-in:
	mkdir -p in
	@num=1; while IFS= read -r line || [ -n "$$line" ]; do \
		echo -n "$$line" > "in/prom$$num"; \
		num=$$((num + 1)); \
	done < src/in.txt

clean:
	rm -rf in out target
