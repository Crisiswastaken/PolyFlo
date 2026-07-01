export function OverlayLoader() {
  return (
    <div className="overlay-loader" aria-hidden="true">
      <svg viewBox="0 0 24 24" className="overlay-loader__svg">
        <circle className="overlay-loader__track" cx="12" cy="12" r="8" fill="none" />
        <circle className="overlay-loader__arc" cx="12" cy="12" r="8" fill="none" />
      </svg>
    </div>
  );
}
