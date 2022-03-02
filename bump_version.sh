
current_version=$(awk '/version/ { print $3 }' Cargo.toml)
sed -i'' '/^const VERSION/ s/"[^"]*"/'"$current_version"'/' src/config.rs

printf "New version written %s\n" "$current_version"
