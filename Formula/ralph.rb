class Ralph < Formula
  desc "Autonomous AI agent loop using Claude Code"
  homepage "https://github.com/kcirtapfromspace/ralph"
  head "https://github.com/kcirtapfromspace/ralph.git", branch: "main"
  license "MIT"

  depends_on "jq"

  def install
    prefix.install Dir["*"]
    bin.install_symlink prefix/"bin/ralph"
  end

  def caveats
    <<~EOS
      Ralph installed! Prerequisites:
        - Claude Code CLI: https://docs.anthropic.com/en/docs/claude-code

      Get started:
        cd your-project
        ralph --init    # Create prd.json template
        ralph           # Run the agent loop

      Docs: https://github.com/kcirtapfromspace/ralph
    EOS
  end

  test do
    assert_match "ralph v", shell_output("#{bin}/ralph --version")
  end
end
