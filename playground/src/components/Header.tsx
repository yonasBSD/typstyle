import { useTheme } from "@/hooks";
import { SampleDocumentSelector } from "./forms/SampleDocumentSelector";
import { DarkModeIcon, GitHubIcon, LightModeIcon } from "./ui/Icons";

interface HeaderProps {
  onSampleSelect: (content: string) => void;
}

export function Header({ onSampleSelect }: HeaderProps) {
  const { theme, toggleTheme } = useTheme();
  return (
    <div className="navbar min-h-12 bg-base-200 shadow">
      <div className="flex-none ml-2">
        <h1 className="text-xl font-bold text-neutral m-1 drop-shadow">
          Typstyle Playground
        </h1>
      </div>

      <div className="flex-1 ml-4">
        <SampleDocumentSelector onSampleSelect={onSampleSelect} />
      </div>

      <div className="flex-none mr-2">
        {/* GitHub Repo Link */}
        <a
          href="https://github.com/typstyle-rs/typstyle"
          target="_blank"
          rel="noopener noreferrer"
          className="btn btn-ghost btn-circle"
          title="View Typstyle on GitHub"
        >
          <GitHubIcon />
        </a>

        {/* Theme Toggle */}
        <button
          type="button"
          onClick={toggleTheme}
          className="btn btn-ghost btn-circle"
          title={`Switch to ${theme === "light" ? "dark" : "light"} mode`}
        >
          {theme === "light" ? <LightModeIcon /> : <DarkModeIcon />}
        </button>
      </div>
    </div>
  );
}
