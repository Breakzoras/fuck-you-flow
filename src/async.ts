/** Bound a backend call so the interface can offer recovery. */
export async function withTimeout<T>(request: Promise<T>, milliseconds: number): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      request,
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error("The app took too long to respond.")), milliseconds);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}
