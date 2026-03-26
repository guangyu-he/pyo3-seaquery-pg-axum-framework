import asyncio

from pyo3_seaquery_pg_axum_framework import AuthUserStruct, test_db_connection_py

if __name__ == "__main__":
    # Python example of calling a rust binding

    async def run():
        await test_db_connection_py()
        user = AuthUserStruct(username="Alice", email="test@example.de")
        user = await user.save_async()
        print(user)
        user.email = "testnew@example.de"
        print(user)

    asyncio.run(run())
