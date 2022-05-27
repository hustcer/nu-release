#!/usr/bin/env nu

# Created: 2022/05/26 19:05:20
# Description:
#   A script to do the github release task, need nushell to be installed.
# REF:
#   1. https://github.com/volks73/cargo-wix

# The binary file to be released
let bin = 'nu'
let os = $env.OS
let target = $env.TARGET
# Repo source dir like `/home/runner/work/nushell/nushell`
let src = $env.GITHUB_WORKSPACE
let dist = $'($env.GITHUB_WORKSPACE)/dist'
let version = (open Cargo.toml | get package.version)

$env

$'Packaging ($bin) v($version) for ($target) in ($src)...'; hr-line -b
if not ('Cargo.lock' | path exists) { cargo generate-lockfile }

$'Start building ($bin)...'; hr-line

# ----------------------------------------------------------------------------
# Build for Windows and macOS
# ----------------------------------------------------------------------------
if $os in ['ubuntu-latest', 'macos-latest'] {
    if $os == 'ubuntu-latest' {
        sudo apt-get install libxcb-composite0-dev
    }
    cargo build --release --all --features=extra,static-link-openssl
}

# ----------------------------------------------------------------------------
# Build for Windows without static-link-openssl feature
# ----------------------------------------------------------------------------
if $os in ['windows-latest'] {
    cargo build --release --all --features=extra
}

# ----------------------------------------------------------------------------
# Prepare for the release archive
# ----------------------------------------------------------------------------
let suffix = if $os == 'windows-latest' { '.exe' } else { '' }
# nu, nu_plugin_* were all included
let executable = $'target/release/($bin)*($suffix)'
$'Current executable file: ($executable)'

cd $src; mkdir $dist;
rm -rf target/release/*.d target/release/nu_pretty_hex*
$'All executable files:'; hr-line -b
ls -f $executable

$'Copying release files...'; hr-line -b
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

    Invoke-WebRequest -Uri 'https://github.com/jftuga/less-Windows/releases/download/less-v562.0/less.exe' -OutFile $'($dist)\less.exe'
    Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/jftuga/less-Windows/master/LICENSE' -OutFile $'($dist)\LICENSE-for-less.txt'

    # Create Windows msi release package
    if (get-env _EXTRA_) == 'msi' {

        let wixRelease = $'($src)/target/wix/($releaseStem).msi'
        $'Start creating Windows msi package...'
        cd $src; hr-line -b
        cargo install cargo-wix --version 0.3.2; cargo wix init
        cargo wix --no-build --nocapture --output $wixRelease
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
