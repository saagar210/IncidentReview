import { faker } from "@faker-js/faker";
import { Factory } from "fishery";

export type User = {
  id: string;
  email: string;
  name: string;
};

export const userFactory = Factory.define<User>(() => ({
  id: faker.string.uuid(),
  email: faker.internet.email(),
  name: faker.person.fullName(),
}));
