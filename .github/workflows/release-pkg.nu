#!/usr/bin/env nu

# Created: 2022/05/26 19:05:20
# Description:
#   A script to do the github release task, need nushell to be installed.
# REF:
#   1. https://github.com/volks73/cargo-wix

# The main binary file to be released
let bin = 'nu'
let os = $env.OS
let target = $env.TARGET
# Repo source dir like `/home/runner/work/nushell/nushell`
let src = $env.GITHUB_WORKSPACE
let flags = $env.TARGET_RUSTFLAGS
let dist = $'($env.GITHUB_WORKSPACE)/output'
let version = (open Cargo.toml | get package.version)

# $env

$'(char nl)Packaging ($bin) v($version) for ($target) in ($src)...'; hr-line -b
if not ('Cargo.lock' | path exists) { cargo generate-lockfile }

$'Start building ($bin)...'; hr-line

# ----------------------------------------------------------------------------
# Build for Ubuntu and macOS
# ----------------------------------------------------------------------------
if $os in ['ubuntu-latest', 'macos-latest'] {
    if $os == 'ubuntu-latest' {
        sudo apt-get install libxcb-composite0-dev
    }
    if ($flags | str trim | empty?) {
        cargo build --release --all --features=extra,static-link-openssl
    } else {
        cargo build --release --all --features=extra,static-link-openssl $flags
    }
}

# ----------------------------------------------------------------------------
# Build for Windows without static-link-openssl feature
# ----------------------------------------------------------------------------
if $os in ['windows-latest'] {
    if ($flags | str trim | empty?) {
        cargo build --release --all --features=extra
    } else {
        cargo build --release --all --features=extra $flags
    }
}

# ----------------------------------------------------------------------------
# Prepare for the release archive
# ----------------------------------------------------------------------------
let suffix = if $os == 'windows-latest' { '.exe' }
# nu, nu_plugin_* were all included
let executable = $'target/release/($bin)*($suffix)'
$'Current executable file: ($executable)'

cd $src; mkdir $dist;
rm -rf target/release/*.d target/release/nu_pretty_hex*
$'(char nl)All executable files:'; hr-line
ls -f $executable

$'(char nl)Copying release files...'; hr-line -b
cp README.release.txt $'($dist)/README.txt'
echo [LICENSE $executable] | each {|it| cp -r $it $dist }
cd $dist; $'Creating release archive...'; hr-line

# ----------------------------------------------------------------------------
# Create a release archive and send it to output for the following steps
# ----------------------------------------------------------------------------
if $os in ['ubuntu-latest', 'macos-latest'] {

    let archive = $'($dist)/($bin)-($version)-($target).tar.gz'
    tar czf $archive *
    print $'archive: ---> ($archive)'; ls $archive
    echo $'::set-output name=archive::($archive)'

} else if $os == 'windows-latest' {

    let releaseStem = $'($bin)-($version)-($target)'

    $'(char nl)Download less related stuffs...'; hr-line
    curl https://github.com/jftuga/less-Windows/releases/download/less-v590/less.exe -o $'($dist)\less.exe'
    curl https://raw.githubusercontent.com/jftuga/less-Windows/master/LICENSE -o $'($dist)\LICENSE-for-less.txt'

    # Create Windows msi release package
    if (get-env _EXTRA_) == 'msi' {

        let wixRelease = $'($src)/target/wix/($releaseStem).msi'
        $'(char nl)Start creating Windows msi package...'
        cd $src; hr-line
        cargo install cargo-wix --version 0.3.2
        cargo wix --no-build --nocapture --package nu --output $wixRelease
        echo $'::set-output name=archive::($wixRelease)'

    } else {

        let archive = $'($dist)/($releaseStem).zip'
        7z a $archive *
        print $'archive: ---> ($archive)';
        let pkg = (ls -f $archive | get name)
        if not ($pkg | empty?) {
            echo $'::set-output name=archive::($pkg | get 0)'
        }
    }
}

# Print a horizontal line marker
def 'hr-line' [
    --blank-line(-b): bool
] {
    print $'(ansi g)---------------------------------------------------------------------------->(ansi reset)'
    if $blank-line { char nl }
}

# Get the specified env key's value or ''
def 'get-env' [
    key: string           # The key to get it's env value
    default: string = ''  # The default value for an empty env
] {
    $env | get -i $key | default $default
}
