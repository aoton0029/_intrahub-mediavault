const originalFetch = window.fetch.bind(window);

function resolveUrl(input: RequestInfo | URL): string {
  if (typeof input === "string") return input;
  if (input instanceof URL) return input.toString();
  return input.url;
}

function resolveMethod(input: RequestInfo | URL, init?: RequestInit): string {
  if (init?.method) return init.method;
  if (input instanceof Request) return input.method;
  return "GET";
}

function parseBody(body: unknown): unknown {
  if (typeof body !== "string") return body;

  try {
    return JSON.parse(body);
  } catch {
    return body;
  }
}

async function describeResponseBody(response: Response): Promise<unknown> {
  const text = await response.clone().text();

  try {
    return text ? JSON.parse(text) : text;
  } catch {
    return text;
  }
}

window.fetch = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
  const method = resolveMethod(input, init);
  const url = resolveUrl(input);
  const label = `[API] ${method} ${url}`;

  console.groupCollapsed(label);
  console.log("request", init?.body ? parseBody(init.body) : undefined);

  try {
    const response = await originalFetch(input, init);
    console.log("response", { status: response.status, body: await describeResponseBody(response) });
    console.groupEnd();
    return response;
  } catch (error) {
    console.log("error", error);
    console.groupEnd();
    throw error;
  }
};
