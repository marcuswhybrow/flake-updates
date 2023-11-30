{
  description = "Bash Script";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }: let 
    pkgs = import nixpkgs { 
      system = "x86_64-linux"; 
      overlays = []; 
    };
    jq = "${pkgs.jq}/bin/jq";
    curl = "${pkgs.curl}/bin/curl";
  in {
    packages.x86_64-linux = {
      nixpkgs-updates = pkgs.writeShellScriptBin "nixpkgs-updates" ''
        flake="''${1:-.}"
        lockfile="$flake/flake.lock"
        nixpkgs="''${2:-nixpkgs}"

        if [ ! -f $lockfile ]; then
          echo "no flake"
          exit 0
        fi

        cd $flake

        xdg_state=''${XDG_STATE_HOME:-"$HOME/.local/state"}
        state_dir="$xdg_state/nixpkgs-updates"
        mkdir --parents $state_dir

        find $state_dir -type f -mmin +60 -delete

        github_url() {
          echo "https://api.github.com/repos/NixOS/nixpkgs/$1"
        }

        github_api() {
          url="$1"
          ${curl} \
            --request GET \
            --silent \
            -H "Accept: application/vnd.github+json" \
            -H "X-GitHub-Api-Version: 2022-11-28" \
            -L \
            $url
        }

        json() {
          file="$1"
          format="$2"
          cat $file | ${jq} --raw-output "$format"
        }

        cur_commit_hash=$(json $lockfile ".nodes.$nixpkgs.locked.rev")
        github_owner=$(json $lockfile ".nodes.$nixpkgs.original.owner")
        github_repo=$(json $lockfile ".nodes.$nixpkgs.original.repo")
        github_ref=$(json $lockfile ".nodes.$nixpkgs.original.ref")

        cur_commit="$state_dir/$github_owner-$github_repo-$cur_commit_hash.json"
        latest_commit="$state_dir/$github_owner-$github_repo-$github_ref.json"

        if [ ! -f $cur_commit ]; then
          url=$(github_url "commits/$cur_commit_hash")
          github_api $url > $cur_commit
        fi

        cur_commit_date=$(json $cur_commit ".commit.committer.date")
        cur_commit_url=$(json $cur_commit ".html_url")

        if [ ! -f $latest_commit ]; then 
          url=$(github_url "commits/$github_ref")
          github_api $url > $latest_commit
        fi

        latest_commit_date=$(json $latest_commit ".commit.committer.date")
        latest_commit_url=$(json $latest_commit ".html_url")
        latest_commit_hash=$(json $latest_commit ".sha")

        if [ "$cur_commit_hash" == "$latest_commit_hash" ]; then 
          exit 0
        fi

        cur_date=`date --date "$cur_commit_date" +%s`
        latest_date=`date --date "$latest_commit_date" +%s`
        minsInHour=60
        secsInMin=60
        hoursInDay=24
        daysInWeek=7
        daysInYear=365

        diff_secs=$(($latest_date-$cur_date))
        diff_mins=$(($diff_secs/$secsInMin))
        diff_hours=$(($diff_mins/$minsInHour))
        diff_days=$(($diff_hours/$hoursInDay))
        diff_weeks=$(($diff_days/$daysInWeek))
        diff_years=$(($diff_days/$daysInYear))

        if (( $diff_years > 0 )); then
          num="$diff_years" 
          unit="year"
        elif (( $diff_weeks > 0 )); then
          num="$diff_weeks" 
          unit="week"
        elif (( $diff_days > 0 )); then
          num="$diff_days" 
          unit="day"
        elif (( $diff_hours > 0 )); then
          num="$diff_hours" 
          unit="hour"
        elif (( $diff_secs > 0 )); then
          num="$diff_secs" 
          unit="sec"
        fi

        if (( $num > 1 )); then 
          unit="''${unit}s"
        fi

        echo " $num $unit"

      '';
      default = self.packages.x86_64-linux.nixpkgs-updates;
    };

    devShells.x86_64-linux.default = pkgs.mkShell {
      buildInputs = [
        pkgs.jq
        pkgs.curl
      ];
    };
  };
}
