# This formula lives in a separate tap repo. This file is a template.
# Copy to caiomestres/homebrew-tap/Casks/gitty.rb when publishing the tap.

cask "gitty" do
  version "0.0.0"
  sha256 "PLACEHOLDER_SHA256"

  url "https://github.com/caiomestres/gitty/releases/download/v#{version}/gitty_#{version}_universal.dmg"
  name "Gitty"
  desc "Workspace synchronization and orchestration for Git repositories"
  homepage "https://github.com/caiomestres/gitty"

  app "Gitty.app"

  zap trash: [
    "~/Library/Application Support/gitty",
    "~/Library/Preferences/com.caiod.gitty.plist",
  ]
end
