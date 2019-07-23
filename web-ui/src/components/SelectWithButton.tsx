import * as React from 'react';

import { Button, Select } from '@material-ui/core';

interface SelectWithButtonProps {
    onChange?: (event: React.ChangeEvent<{ name?: string | undefined, value: unknown }>, child: React.ReactNode) => void;
    onClick?: (event: React.MouseEvent<HTMLButtonElement>) => void;
    value?: Array<string | number | boolean> | string | number | boolean;
    children?: React.ReactNode;
    buttonText?: React.ReactNode;
    style?: React.CSSProperties;
}

export const SelectWithButton = (props: SelectWithButtonProps) => {
    return (
        <>
            <Select
                style={{ width: 200, ...props.style }}
                value={props.value}
                onChange={props.onChange}
            >
                {props.children}
            </Select>
            <Button style={{ marginLeft: 20 }} variant="outlined" onClick={props.onClick}>
                {props.buttonText}
            </Button>
        </>
    )
}