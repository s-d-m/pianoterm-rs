build clean run:
	cargo "$@"

# # Failing a build that has warnings and a list of warnings to activate
# https://users.rust-lang.org/t/how-to-fail-a-build-with-warnings/2687/8
# https://github.com/rust-unofficial/patterns/blob/master/anti_patterns/deny-warnings.md

clippy:
	rustup run nightly cargo clippy -- \
		--warn invalid_upcast_comparisons \
		--warn items_after_statements \
		--warn mut_mut \
		--warn mutex_integer \
		--warn nonminimal_bool \
		--warn option_map_unwrap_or \
		--warn option_map_unwrap_or_else \
		--warn option_unwrap_used \
		--warn print_with_newline \
		--warn pub_enum_variant_names \
		--warn result_unwrap_used \
		--warn stutter \
		--warn unicode_not_nfc \
		--warn unseparated_literal_suffix \
		--warn used_underscore_binding \
		--warn wrong_pub_self_convention \



.PHONY: all clean clippy run
