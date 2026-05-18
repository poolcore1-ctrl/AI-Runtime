import dayjs from "dayjs";

type User = {
  name: string;
  email: string;
};

const user: User = {
  name: "ASOS",
  email: "asos@example.com",
};

console.log(user.email);
console.log(dayjs().format());
