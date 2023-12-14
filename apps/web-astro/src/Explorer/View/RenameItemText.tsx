import clsx from 'clsx';
import { JSX } from 'solid-js';
import { getExplorerItemData, useRspcLibraryContext, type ExplorerItem } from '@sd/client';

import { useExplorerContext } from '../Context';
// import { toast } from '@sd/ui';
// import { useIsDark } from '~/hooks';

// import { useExplorerContext } from '../Context';
import { RenameTextBox, type RenameTextBoxProps } from '../FilePath/RenameTextBox';

// import { useQuickPreviewStore } from '../QuickPreview/store';
// import { useExplorerStore } from '../store';

interface Props extends Pick<RenameTextBoxProps, 'idleClassName' | 'lines'> {
	item: ExplorerItem;
	allowHighlight?: boolean;
	style?: JSX.CSSProperties;
	highlight?: boolean;
	selected?: boolean;
}

export const RenamableItemText = ({ allowHighlight = true, ...props }: Props) => {
	const isDark = true; //useIsDark();
	const rspc = useRspcLibraryContext();

	const explorer = useExplorerContext();

	// const quickPreviewStore = useQuickPreviewStore();

	const itemData = () => getExplorerItemData(props.item);

	let ref: HTMLDivElement;

	// const renameFile = useLibraryMutation(['files.renameFile'], {
	// 	onError: () => reset(),
	// 	onSuccess: () => rspc.queryClient.invalidateQueries(['search.paths'])
	// });

	// const renameEphemeralFile = useLibraryMutation(['ephemeralFiles.renameFile'], {
	// 	onError: () => reset(),
	// 	onSuccess: () => rspc.queryClient.invalidateQueries(['search.paths'])
	// });

	// const renameLocation = useLibraryMutation(['locations.update'], {
	// 	onError: () => reset(),
	// 	onSuccess: () => rspc.queryClient.invalidateQueries(['search.paths'])
	// });

	const reset = () => {
		const fullName = itemData().fullName;
		if (!ref || !fullName) return;
		ref.innerText = fullName;
	};

	const handleRename = async (newName: string) => {
		try {
			// switch (props.item.type) {
			// 	case 'Location': {
			// 		const locationId = props.item.item.id;
			// 		if (!locationId) throw new Error('Missing location id');
			// 		await renameLocation.mutateAsync({
			// 			id: locationId,
			// 			path: null,
			// 			name: newName,
			// 			generate_preview_media: null,
			// 			sync_preview_media: null,
			// 			hidden: null,
			// 			indexer_rules_ids: []
			// 		});
			// 		break;
			// 	}
			// 	case 'Path':
			// 	case 'Object': {
			// 		const filePathData = getIndexedItemFilePath(props.item);
			// 		if (!filePathData) throw new Error('Failed to get file path object');
			// 		const { id, location_id } = filePathData;
			// 		if (!location_id) throw new Error('Missing location id');
			// 		await renameFile.mutateAsync({
			// 			location_id: location_id,
			// 			kind: {
			// 				One: {
			// 					from_file_path_id: id,
			// 					to: newName
			// 				}
			// 			}
			// 		});
			// 		break;
			// 	}
			// 	case 'NonIndexedPath': {
			// 		const ephemeralFile = getEphemeralPath(props.item);
			// 		if (!ephemeralFile) throw new Error('Failed to get ephemeral file object');
			// 		renameEphemeralFile.mutate({
			// 			kind: {
			// 				One: {
			// 					from_path: ephemeralFile.path,
			// 					to: newName
			// 				}
			// 			}
			// 		});
			// 		break;
			// 	}
			// 	default:
			// 		throw new Error('Invalid explorer item type');
			// }
		} catch (e) {
			reset();
			// toast.error({
			// 	title: `Could not rename ${itemData.fullName} to ${newName}`,
			// 	body: `Error: ${e}.`
			// });
		}
	};

	const disabled = !props.selected;
	//||
	// explorerStore.drag?.type === 'dragging' ||
	// !explorer ||
	// explorer.selectedItems.size > 1 ||
	// quickPreviewStore.open ||
	// props.item.type === 'SpacedropPeer';

	return (
		<RenameTextBox
			ref={ref!}
			name={itemData().fullName ?? itemData.name ?? ''}
			disabled={disabled}
			onRename={handleRename}
			class={clsx(
				'font-medium',
				(props.selected || props.highlight) &&
					allowHighlight && ['bg-accent', !isDark && 'text-white']
			)}
			style={props.style}
			lines={props.lines}
			idleClassName={props.idleClassName}
		/>
	);
};