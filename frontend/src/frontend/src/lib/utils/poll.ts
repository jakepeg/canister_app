// src/frontend/src/lib/utils/poll.ts

/**
 * Configuration for the poll function.
 * @template T The expected type of the result when polling is successful.
 */
export interface PollConfig<T> {
	/** The asynchronous function to execute in each polling attempt. */
	fn: () => Promise<T>;
	/** A function that validates the result. Polling stops if it returns true. */
	validate: (result: T) => boolean;
	/** An optional function to determine if polling should exit based on an error. */
	shouldExit?: (error: unknown) => boolean;
	/** The interval between polling attempts in milliseconds. Defaults to 5000 (5 seconds). */
	interval?: number;
	/** The maximum time to poll in milliseconds before timing out. Defaults to 300000 (5 minutes). */
	timeout?: number;
}

/**
 * Polls an asynchronous function until a condition is met or a timeout occurs.
 * @template T The expected type of the result.
 * @param {PollConfig<T>} config The polling configuration.
 * @returns {Promise<T>} A promise that resolves with the validated result or rejects on timeout or specific errors.
 */
export const poll = <T>({
	fn,
	validate,
	shouldExit = () => false,
	interval = 5000,
	timeout = 300000
}: PollConfig<T>): Promise<T> =>
	new Promise((resolve, reject) => {
		const startTime = Date.now();

		const executePoll = async () => {
			if (Date.now() - startTime > timeout) {
				reject(new Error('Polling timed out'));
				return;
			}

			try {
				const result = await fn();
				if (validate(result)) {
					resolve(result);
					return;
				}
				// Result not valid yet, poll again
				console.log('Polling condition not met, retrying...');
				setTimeout(executePoll, interval);
			} catch (error: unknown) {
				console.error('Polling error:', error);
				if (shouldExit(error)) {
					console.log('Exiting poll due to specific error.');
					reject(error); // Exit polling on specific errors
					return;
				}
				// Continue polling on other errors
				console.log('Continuing poll despite error...');
				setTimeout(executePoll, interval);
			}
		};

		console.log('Starting poll...');
		executePoll();
	});
