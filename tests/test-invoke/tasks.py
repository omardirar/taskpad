"""Sample Invoke tasks for testing taskpad."""
from invoke import task


@task
def build(c):
    """Build the project"""
    print("Building project...")


@task
def test(c):
    """Run tests"""
    print("Running tests...")


@task
def deploy(c):
    """Deploy to production"""
    print("Deploying...")


@task
def clean(c):
    """Clean build artifacts"""
    print("Cleaning...")
