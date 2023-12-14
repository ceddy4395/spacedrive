import { type ProcedureDef } from '@rspc/client';
import { AlphaRSPCError, initRspc } from '@rspc/client/v2';
import { createReactQueryHooks, type Context } from '@rspc/solid';
import { QueryClient } from '@tanstack/react-query';
import { createContext, useContext, type ParentProps } from 'solid-js';
import { match, P } from 'ts-pattern';
import { currentLibraryCache, type LibraryArgs, type Procedures } from '@sd/client';

type NonLibraryProcedure<T extends keyof Procedures> =
	| Exclude<Procedures[T], { input: LibraryArgs<any> }>
	| Extract<Procedures[T], { input: never }>;

type LibraryProcedures<T extends keyof Procedures> = Exclude<
	Extract<Procedures[T], { input: LibraryArgs<any> }>,
	{ input: never }
>;

type StripLibraryArgsFromInput<
	T extends ProcedureDef,
	NeverOverNull extends boolean
> = T extends any
	? T['input'] extends LibraryArgs<infer E>
		? {
				key: T['key'];
				input: NeverOverNull extends true ? (E extends null ? never : E) : E;
				result: T['result'];
		  }
		: never
	: never;

type NonLibraryProceduresDef = {
	queries: NonLibraryProcedure<'queries'>;
	mutations: NonLibraryProcedure<'mutations'>;
	subscriptions: NonLibraryProcedure<'subscriptions'>;
};

export type LibraryProceduresDef = {
	queries: StripLibraryArgsFromInput<LibraryProcedures<'queries'>, true>;
	mutations: StripLibraryArgsFromInput<LibraryProcedures<'mutations'>, false>;
	subscriptions: StripLibraryArgsFromInput<LibraryProcedures<'subscriptions'>, true>;
};

const context = createContext<Context<LibraryProceduresDef>>({} as any);

export const useRspcLibraryContext = () => useContext(context);

export const rspc = initRspc<Procedures>({
	links: globalThis.rspcLinks
});
export const rspc2 = initRspc<Procedures>({
	links: globalThis.rspcLinks
}); // TODO: Removing this?

export const nonLibraryClient = rspc.dangerouslyHookIntoInternals<NonLibraryProceduresDef>();
// @ts-expect-error // TODO: Fix
const nonLibraryHooks = createReactQueryHooks<NonLibraryProceduresDef>(nonLibraryClient, {
	// context // TODO: Shared context
});

export const libraryClient = rspc2.dangerouslyHookIntoInternals<LibraryProceduresDef>({
	mapQueryKey: (keyAndInput) => {
		const libraryId = currentLibraryCache.id;
		if (libraryId === null)
			throw new Error('Attempted to do library operation with no library set!');
		return [keyAndInput[0], { library_id: libraryId, arg: keyAndInput[1] ?? null }];
	}
});
const libraryHooks = createReactQueryHooks<LibraryProceduresDef>(libraryClient, {
	// @ts-expect-error // TODO: idk
	context
});

// TODO: Allow both hooks to use a unified context -> Right now they override each others local state
export function RspcProvider({ queryClient, children }: ParentProps<{ queryClient: QueryClient }>) {
	return (
		<libraryHooks.Provider client={libraryClient as any} queryClient={queryClient}>
			<nonLibraryHooks.Provider client={nonLibraryClient as any} queryClient={queryClient}>
				{children as any}
			</nonLibraryHooks.Provider>
		</libraryHooks.Provider>
	);
}

export const createBridgeQuery = nonLibraryHooks.createQuery;
export const createBridgeMutation = nonLibraryHooks.createMutation;
export const createBridgeSubscription = nonLibraryHooks.createSubscription;
export const createLibraryQuery = libraryHooks.createQuery;
export const createLibraryMutation = libraryHooks.createMutation;
export const createLibrarySubscription = libraryHooks.createSubscription;

export function useInvalidateQuery() {
	const context = nonLibraryHooks.useContext();
	createBridgeSubscription(() => ['invalidation.listen'] as const, {
		onData: (ops) => {
			for (const op of ops) {
				match(op)
					.with({ type: 'single', data: P.select() }, (op) => {
						let key: any[] = [op.key];
						if (op.arg !== null) {
							key = key.concat(op.arg);
						}

						if (op.result !== null) {
							context.queryClient.setQueryData(key, op.result);
						} else {
							context.queryClient.invalidateQueries(key);
						}
					})
					.with({ type: 'all' }, (op) => {
						context.queryClient.invalidateQueries();
					})
					.exhaustive();
			}
		}
	});
}

// TODO: Remove/fix this when rspc typesafe errors are working
export function extractInfoRSPCError(error: unknown) {
	if (!(error instanceof AlphaRSPCError)) return null;
	return error;
}