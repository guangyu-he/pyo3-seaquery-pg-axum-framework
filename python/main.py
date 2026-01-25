import asyncio

from pydantic import BaseModel
from pyo3_seaquery_pg_axum_framework import AuthUserStruct, test_db_connection_py


class User(BaseModel):
    id: int
    name: str
    email: str

    def greet(self) -> str:
        return f"Hello, {self.name}"


if __name__ == "__main__":
    async def run():
        await test_db_connection_py()
        user = AuthUserStruct(username="Alice", email="test@example.de")
        user = await user.save()
        print(user)
        user.email = "testnew@example.de"
        print(user)


    asyncio.run(run())
