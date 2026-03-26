from pydantic import BaseModel


# Rust will call this class
class User(BaseModel):
    id: int
    name: str
    email: str

    def greet(self) -> str:
        return f"Hello, {self.name}"


# Rust will call this function
def hello(name: str) -> str:
    return f"Hello, {name}!"
