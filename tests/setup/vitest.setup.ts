import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach, beforeEach } from "vitest";
import { faker } from "@faker-js/faker";

beforeEach(() => {
  faker.seed(12345);
});

afterEach(() => {
  faker.seed();
  cleanup();
});
