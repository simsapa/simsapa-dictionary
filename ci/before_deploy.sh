echo "=== before_deploy.sh ==="

os=$TRAVIS_OS_NAME
echo "OS: '$os'"

if [[ "$os" == "linux" ]]; then
	cp ./target/release/simsapa_dictionary ./simsapa_dictionary_linux
elif [[ "$os" == "osx" ]]; then
	cp ./target/release/simsapa_dictionary ./simsapa_dictionary_osx
fi
