# FileMaker-Export-Exploder Template

This template contains an example folder structure to run a FileMaker XML export, explode it with the
FM-Export-Exploder and commit the results to git repositories.

## Setup

1. Download FM-Export-Exploder binaries to `bin`.
2. Create `Run_XML_Export.fmp12`, which runs a script on opening. The script should do automatically export your
   solution file(s) as XML using
   the [Save Copy as XML command](https://help.claris.com/en/pro-help/content/save-a-copy-as-xml.html). __Tip:__ You can
   apply the command to any opened FileMaker file that is open with full access by window name.
3. Run `init_repos.bash` once to initialize the git repositories for the exploded XML export.
4. Run `run.bash` to perform an export, explode and git commit.

## Install service

Use the `service/launchd_service.plist` template to create a service that runs the script daily.

Navigate to the `~/Library/LaunchAgents` directory from your current user. You can access this directory by opening
Finder, choosing "Go to Folder" from menu and entering `~/Library/LaunchAgents`.

Save the `launchd_service.plist` file into this directory. If the directory doesn't exist, you may need to create it.

1. After saving the file, it needs to be edited to ensure that the username and paths are correct. Open the
   `launchd_service.plist` file in a text editor such as TextEdit or VSCode.
2. Locate the placeholders for the username and paths within the file and replace them.
3. Save the changes to the launchd_service.plist file.
4. To load the service file, run the command `launchctl load ~/Library/LaunchAgents/launchd_service.plist`.
