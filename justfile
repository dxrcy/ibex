publish:
	cargo clippy &&\
	echo &&\
	echo   '-------------------------' &&\
	printf 'Continue with publishing? ' &&\
	read -r &&\
	echo '$ cargo publish -p ibex_core' &&\
	echo '$ cargo publish -p ibex_macros' &&\
	echo '$ cargo publish -p ibex'

