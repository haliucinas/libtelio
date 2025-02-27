import asyncio
from asyncio import Future
from typing import Union, Coroutine, TypeVar, Any

# This modules defines standardized waiting categories for tests. Some tasks are expected
# to finish very quickly, hence the waiting time is very short (0.1 seconds). Other tasks
# are expected to take more time, so the waiting time is much longer (5 seconds).

T = TypeVar("T")


async def wait_short(coroutine: Union[Coroutine[Any, Any, T], Future]) -> T:
    """Wait for 0.1 seconds"""
    return await asyncio.wait_for(coroutine, 0.1)


async def wait_normal(coroutine: Union[Coroutine[Any, Any, T], Future]) -> T:
    """Wait for 1 second"""
    return await asyncio.wait_for(coroutine, 1)


async def wait_long(coroutine: Union[Coroutine[Any, Any, T], Future]) -> T:
    """Wait for 5 seconds"""
    return await asyncio.wait_for(coroutine, 5)


async def wait_lengthy(coroutine: Union[Coroutine[Any, Any, T], Future]) -> T:
    """Wait for 30 seconds"""
    return await asyncio.wait_for(coroutine, 30)


async def wait_defined(
    coroutine: Union[Coroutine[Any, Any, T], Future], defined_wait
) -> T:
    """Wait for defined seconds"""
    return await asyncio.wait_for(coroutine, defined_wait)
