// src/lib/services/idleManager.ts

export class IdleManager {
  private timeoutId: ReturnType<typeof setTimeout> | null = null;
  private readonly activityEvents = ['mousemove', 'keydown', 'click', 'scroll'];
  private readonly idleTimeout: number;
  private readonly onIdle: () => void;

  constructor(private authClient: any, options: { idleTimeout: number; onIdle: () => void }) {
    this.idleTimeout = options.idleTimeout;
    this.onIdle = options.onIdle;
  }

  start() {
    this.resetTimer();
    this.activityEvents.forEach((event) => {
      window.addEventListener(event, this.resetTimer);
    });
  }

  stop() {
    if (this.timeoutId) clearTimeout(this.timeoutId);
    this.activityEvents.forEach((event) => {
      window.removeEventListener(event, this.resetTimer);
    });
  }

  private resetTimer = () => {
    if (this.timeoutId) clearTimeout(this.timeoutId);

    this.timeoutId = setTimeout(() => {
      this.stop(); // Stop listening once user is idle
      this.onIdle(); // Call the provided logout function
    }, this.idleTimeout);
  };
}
