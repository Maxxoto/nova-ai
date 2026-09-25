export default function ReadAloudButton({
  reading,
  onRead,
  onStop,
}: {
  reading: boolean;
  onRead: () => void;
  onStop: () => void;
}) {
  return (
    <button
      type="button"
      aria-pressed={reading}
      onClick={reading ? onStop : onRead}
      className="inline-flex h-[26px] flex-none items-center gap-1.5 whitespace-nowrap rounded-[10px] px-2.5 font-ui text-[13px] font-semibold text-muted-foreground transition-colors duration-[120ms] ease-[cubic-bezier(.4,0,.2,1)] hover:bg-muted hover:text-foreground focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-ring"
    >
      <svg
        viewBox="0 0 24 24"
        className="h-3.5 w-3.5"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.7"
        strokeLinecap="round"
        aria-hidden="true"
      >
        <path d="M11 5 6 9H3v6h3l5 4Z" />
        <path d="M16 9a4 4 0 0 1 0 6" />
      </svg>
      <span>{reading ? "Stop reading" : "Read aloud"}</span>
    </button>
  );
}
